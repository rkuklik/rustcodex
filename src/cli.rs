use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;
use clap::ValueHint;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    /// Language to target as host
    #[arg(short, long, env)]
    pub target: TargetLanguage,

    /// Manually specified source files
    #[arg(short, long = "file", env, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,

    #[arg(short, long, env)]
    /// Get files for source language
    pub source: Option<SourceLanguage>,

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

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum TargetLanguage {
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum SourceLanguage {
    Rust,
}
