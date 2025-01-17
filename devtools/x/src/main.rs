// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use chrono::Local;
use clap::Parser;
use env_logger::{self, fmt::Color};
use log::Level;
use std::{boxed::Box, io::Write};

pub mod bench {
    pub use x::bench::*;
}
pub mod build {
    pub use x::build::*;
}
pub mod check {
    pub use x::check::*;
}
pub mod changed_since {
    pub use x::changed_since::*;
}
pub mod clippy {
    pub use x::clippy::*;
}
pub mod config {
    pub use x::config::*;
}
pub mod context {
    pub use x::context::*;
}
pub mod diff_summary {
    pub use x::diff_summary::*;
}
pub mod fix {
    pub use x::fix::*;
}
pub mod fmt {
    pub use x::fmt::*;
}
pub mod generate_summaries {
    pub use x::generate_summaries::*;
}
pub mod installer {
    pub use x::installer::*;
}

pub mod lint {
    pub use x::lint::*;
}
pub mod playground {
    pub use x::playground::*;
}
pub mod test;

pub mod nextest {
    pub use x::nextest::*;
}

pub mod cargo {
    pub use x::cargo::*;
}
pub mod tools {
    pub use x::tools::*;
}

pub mod utils;

type Result<T> = anyhow::Result<T>;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(name = "bench")]
    /// Run `cargo bench`
    Bench(bench::Args),
    #[clap(name = "build")]
    /// Run `cargo build`
    // the argument must be Boxed due to it's size and clippy (it's quite large by comparison to others.)
    Build(Box<build::Args>),
    #[clap(name = "check")]
    /// Run `cargo check`
    Check(check::Args),
    /// List packages changed since merge base with the given commit
    ///
    /// Note that this compares against the merge base (common ancestor) of the specified commit.
    /// For example, if origin/master is specified, the current working directory will be compared
    /// against the point at which it branched off of origin/master.
    #[clap(name = "changed-since")]
    ChangedSince(changed_since::Args),
    #[clap(name = "clippy")]
    /// Run `cargo clippy`
    Clippy(clippy::Args),
    #[clap(name = "fix")]
    /// Run `cargo fix`
    Fix(fix::Args),
    #[clap(name = "fmt")]
    /// Run `cargo fmt`
    Fmt(fmt::Args),
    #[clap(name = "test")]
    /// Run tests
    Test(test::Args),
    #[clap(name = "nextest")]
    /// Run tests with new test runner
    Nextest(nextest::Args),
    #[clap(name = "tools")]
    /// Run tests
    Tools(tools::Args),
    #[clap(name = "lint")]
    /// Run lints
    Lint(lint::Args),
    /// Run playground code
    Playground(playground::Args),
    #[clap(name = "generate-summaries")]
    /// Generate build summaries for important subsets
    GenerateSummaries(generate_summaries::Args),
    #[clap(name = "diff-summary")]
    /// Diff build summaries for important subsets
    DiffSummary(diff_summary::Args),
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let color = match record.level() {
                Level::Warn => Color::Yellow,
                Level::Error => Color::Red,
                _ => Color::Green,
            };

            let mut level_style = buf.style();
            level_style.set_color(color).set_bold(true);

            writeln!(
                buf,
                "{:>12} [{}] - {}",
                level_style.value(record.level()),
                Local::now().format("%T%.3f"),
                record.args()
            )
        })
        .init();

    let args = Args::parse();
    let xctx = context::XContext::with_project_root(utils::project_root())?;
    match args.cmd {
        Command::Tools(args) => tools::run(args, xctx),
        Command::Test(args) => test::run(args, xctx),
        Command::Nextest(args) => nextest::run(args, xctx),
        Command::Build(args) => build::run(args, xctx),
        Command::ChangedSince(args) => changed_since::run(args, xctx),
        Command::Check(args) => check::run(args, xctx),
        Command::Clippy(args) => clippy::run(args, xctx),
        Command::Fix(args) => fix::run(args, xctx),
        Command::Fmt(args) => fmt::run(args, xctx),
        Command::Bench(args) => bench::run(args, xctx),
        Command::Lint(args) => lint::run(args, xctx),
        Command::Playground(args) => playground::run(args, xctx),
        Command::GenerateSummaries(args) => generate_summaries::run(args, xctx),
        Command::DiffSummary(args) => diff_summary::run(args, xctx),
    }
}
