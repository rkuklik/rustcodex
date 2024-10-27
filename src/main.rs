use std::fs::File;
use std::io::stdin;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;
use rustcodex::cli::Language;
use rustcodex::host::Python;
use rustcodex::source::MergedSources;
use rustcodex::source::Rust;
use rustcodex::source::Source;
use terminator::Verbosity;

use rustcodex::cli::Cli;

fn main() -> Result<(), terminator::Terminator> {
    terminator::Config::new()
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
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
    };

    // Ensure that payload is readable
    payload
        .fill_buf()
        .with_context(|| "input must be readable")?;

    let mut output: Box<dyn Write> = match output {
        None => Box::new(BufWriter::new(stdout().lock())),
        Some(file) => Box::new(BufWriter::new(
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(file)?,
        )),
    };

    let template = match target {
        Language::Python => Box::new(Python::new(payload, source, compress)),
        Language::Rust => Err(anyhow!("`Rust` doesn't have runner"))?,
    };

    output.write_fmt(format_args!("{template}"))?;

    Ok(())
}
