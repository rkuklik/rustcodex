use std::path::PathBuf;

use clap::Parser;
use clap::ValueHint;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    // `Cli` is inlined into build script for completion generation, but `Language`
    // is generated from it. So target field is only included if it is outside the
    // build script. `generated` is set by `build.rs`, which must be kept in sync.
    /// Output language
    #[cfg(generated)]
    #[arg(short, long, env)]
    pub target: crate::lang::Language,

    /// Paths to UTF-8 encoded source files
    #[arg(short, long, env, value_hint = ValueHint::FilePath, num_args = 0..)]
    pub source: Vec<PathBuf>,

    /// Input file with `-` for stdin
    #[arg(short, long, env, value_hint = ValueHint::FilePath)]
    pub input: PathBuf,

    /// Output file with `-` for stdout
    #[arg(short, long, env)]
    pub output: PathBuf,
}

impl Cli {
    #[must_use]
    pub fn parse() -> Self {
        Parser::parse()
    }
}
