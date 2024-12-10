use std::env::set_var;
use std::env::var_os;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

use anyhow::Context;
use anyhow::Error;
use rustcodex::cli::Cli;
use rustcodex::inout::Input;
use rustcodex::inout::Output;
use rustcodex::lang::Data;
use rustcodex::lang::Template;
use rustcodex::source::SourceFile;

static BACKTRACE: &str = "RUST_BACKTRACE";

fn main() -> Result<(), Error> {
    if var_os(BACKTRACE).is_none() {
        // SAFETY: there are no other threads running before this function is called
        unsafe {
            // Always capture a backtrace
            set_var(BACKTRACE, "full");
        }
    }

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
