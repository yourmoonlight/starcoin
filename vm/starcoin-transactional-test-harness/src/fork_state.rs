// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::HashValue;
use anyhow::{anyhow, Result};
use move_core_types::account_address::AccountAddress;
use starcoin_state_api::{
    ChainStateAsyncService, ChainStateReader, StateNodeStore, StateView, StateWithProof,
};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::state_node::StateStorage;
use starcoin_storage::storage::{CodecKVStore, CodecWriteBatch, StorageInstance};

use futures::executor::block_on;
use starcoin_rpc_api::state::StateApiClient;
use starcoin_state_tree::StateNode;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::AccountStateSet;

pub struct MockStateNodeStore {
    local_storage: StateStorage,
    remote: Arc<StateApiClient>,
}

impl MockStateNodeStore {
    pub fn new(remote: Arc<StateApiClient>) -> Result<Self> {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = StateStorage::new(storage_instance);

        Ok(Self {
            local_storage: storage,
            remote,
        })
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        match self.local_storage.get(*hash)? {
            Some(sn) => Ok(Some(sn)),
            None => {
                let blob =
                    block_on(async move { self.remote.get_state_node_by_node_hash(*hash).await })
                        .map(|res| res.map(StateNode))
                        .map_err(|e| anyhow!("{}", e))?;

                // Put result to local storage to accelerate the following getting.
                if let Some(node) = blob.clone() {
                    self.put(*hash, node)?;
                };
                Ok(blob)
            }
        }
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.local_storage.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.local_storage.write_batch(batch)
    }
}

#[derive(Clone)]
pub struct MockChainStateAsyncService {
    state_store: Arc<dyn StateNodeStore>,
    root: Arc<Mutex<HashValue>>,
}

impl MockChainStateAsyncService {
    pub fn new(state_store: Arc<dyn StateNodeStore>, root: Arc<Mutex<HashValue>>) -> Self {
        Self { state_store, root }
    }

    fn state_db(&self) -> ChainStateDB {
        let root = self.root.lock().unwrap();
        ChainStateDB::new(self.state_store.clone(), Some(*root))
    }
}

#[async_trait::async_trait]
impl ChainStateAsyncService for MockChainStateAsyncService {
    async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>> {
        self.state_db().get(&access_path)
    }

    async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProof> {
        self.state_db().get_with_proof(&access_path)
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>> {
        self.state_db().get_account_state(&address)
    }
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<Option<AccountStateSet>> {
        match state_root {
            Some(root) => {
                let reader = self.state_db().fork_at(root);
                reader.get_account_state_set(&address)
            }
            None => self.state_db().get_account_state_set(&address),
        }
    }
    async fn state_root(self) -> Result<HashValue> {
        Ok(self.state_db().state_root())
    }

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let reader = self.state_db().fork_at(state_root);
        reader.get_with_proof(&access_path)
    }

    async fn get_account_state_by_root(
        self,
        account_address: AccountAddress,
        state_root: HashValue,
    ) -> Result<Option<AccountState>> {
        let reader = self.state_db().fork_at(state_root);
        reader.get_account_state(&account_address)
    }
}