use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

use anyhow::Context;
use rustcodex::cli::Cli;
use rustcodex::inout::Input;
use rustcodex::inout::Output;
use rustcodex::lang::Data;
use rustcodex::lang::Template;
use rustcodex::source::SourceFile;
use terminator::Config;
use terminator::Terminator;
use terminator::Verbosity;

fn main() -> Result<(), Terminator> {
    // nicer backtrace and error fmt
    Config::new()
        .verbosity(Verbosity::error().unwrap_or(Verbosity::Medium))
        .install()?;

    let Cli {
        target,
        source,
        input,
        output,
    } = Cli::parse();

    let sources = SourceFile::load(source).context("failed to load source files")?;

    let mut payload = Vec::new();
    Input::parse(input)
        .context("opening input file failed")?
        .read_to_end(&mut payload)
        .context("input isn't readable")?;

    let output = Output::parse(output).context("opening output file failed")?;

    let template = Template::new(Data::new(&payload, &sources)).transform(target);

    write!(BufWriter::new(output), "{template}")?;

    Ok(())
}
