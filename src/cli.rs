use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, arg_required_else_help = true)]
pub struct Cli {
    /// Language to target as host
    #[arg(short, long, env)]
    pub target: Language,

    /// Manually specified source files
    #[arg(short, long = "file", env)]
    pub files: Vec<PathBuf>,

    #[arg(short, long, env)]
    /// Get files for source language
    pub source: Option<Language>,

    /// Compress the binary
    #[arg(short, long, env)]
    pub compress: bool,

    /// Input (defaults to stdin)
    #[arg(short, long, env)]
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
pub enum Language {
    Python,
    Rust,
}

impl Language {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Python => "py",
            Self::Rust => "rs",
        }
    }
}
