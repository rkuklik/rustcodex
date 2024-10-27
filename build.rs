use std::fs;
use std::io::Error;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell;

const DIR: &str = "completions";
const APP: &str = "rustcodex";

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    if !fs::exists(DIR)? {
        fs::create_dir(DIR)?;
    }

    let mut app = Cli::command();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, &mut app, APP, DIR)?;
    }

    println!("cargo::rerun-if-changed=src/cli.rs");

    Ok(())
}
