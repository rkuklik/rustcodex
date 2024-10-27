use std::fmt::Display;
use std::fs::File;
use std::io::stdin;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;
use rustcodex::cli::Cli;
use rustcodex::cli::Language;
use rustcodex::host::Python;
use rustcodex::source::MergedSources;
use rustcodex::source::Rust;
use rustcodex::source::Source;
use terminator::Config;
use terminator::Terminator;
use terminator::Verbosity;

fn main() -> Result<(), Terminator> {
    Config::new()
        .verbosity(Verbosity::error().unwrap_or(Verbosity::Medium))
        .install()?;

    let Cli {
        target,
        files,
        source,
        compress,
        input,
        output,
    } = Cli::parse();

    let source: Box<dyn Source> = match source {
        None => Box::new(files),
        Some(source) => Box::new(MergedSources {
            first: files,
            second: match source {
                Language::Rust => Rust,
                Language::Python => Err(anyhow!("`Python` doesn't have source detector"))?,
            },
        }),
    };

    let mut payload: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(stdin().lock())),
        Some(file) => Box::new(BufReader::new(
            File::options()
                .read(true)
                .write(false)
                .truncate(false)
                .create(false)
                .open(file)
                .context("opening input file failed")?,
        )),
    };

    // Ensure that payload is readable
    payload.fill_buf().context("input isn't readable")?;

    let mut output: Box<dyn Write> = match output {
        None => Box::new(BufWriter::new(stdout().lock())),
        Some(file) => Box::new(BufWriter::new(
            File::options()
                .read(false)
                .write(true)
                .truncate(true)
                .create(true)
                .open(file)
                .context("opening output file failed")?,
        )),
    };

    let template: Box<dyn Display> = match target {
        Language::Python => Box::new(Python::new(payload, source, compress)),
        Language::Rust => Err(anyhow!("`Rust` doesn't have runner"))?,
    };

    write!(output, "{template}")?;

    Ok(())
}
