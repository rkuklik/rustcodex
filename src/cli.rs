use std::path::PathBuf;

use clap::Parser;
use clap::ValueHint;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    /// Language to target as host
    // `Cli` is inlined into build script, but `Language` is generated from it.
    #[cfg(nonrecursive)]
    #[arg(short, long, env)]
    pub target: crate::lang::Language,

    /// Manually specified source files
    #[arg(short, long = "source", env, value_hint = ValueHint::FilePath, num_args = 0..)]
    pub sources: Vec<PathBuf>,

    /// Input (defaults to stdin)
    #[arg(short, long, env, value_hint = ValueHint::FilePath)]
    pub input: Option<PathBuf>,

    /// Output (defaults to stdout)
    #[arg(short, long, env)]
    pub output: Option<PathBuf>,
}

impl Cli {
    #[must_use]
    pub fn parse() -> Self {
        Parser::parse()
    }
}
