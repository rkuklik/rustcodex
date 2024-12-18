//! Runtime support for template instantiation

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::str::from_utf8;

use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::source::SourceFile;

/// Adapter between `io::Write` and `fmt::Write`
struct IoCompat<'m, 'f> {
    f: &'m mut Formatter<'f>,
}

impl Write for IoCompat<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.f
            .write_str(from_utf8(buf).map_err(|error| Error::new(ErrorKind::InvalidData, error))?)
            .map_err(Error::other)
            .map(|()| buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Writes the payload as compressed base64 encoded string
struct Compressor<'a> {
    payload: &'a [u8],
}

impl Display for Compressor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let iocompat = IoCompat { f };
        let error = "payload builder wrote invalid data";
        let mut writer = GzEncoder::new(
            EncoderWriter::new(iocompat, &BASE64_STANDARD),
            Compression::best(),
        );
        writer.write_all(self.payload).expect(error);
        writer.finish().expect(error);
        Ok(())
    }
}

/// Formatter for to-be inlined source code
///
/// `start` and `end` are start and end of comment provided by template
struct CodeInliner<'s> {
    start: &'static str,
    end: &'static str,
    files: &'s [SourceFile],
}

impl Display for CodeInliner<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { start, end, files } = *self;

        macro_rules! s {
            ($($arg:tt)*) => {
                writeln!(f, "{start}{}{end}", format_args!($($arg)*))?;
            };
        }

        s!("Generated by `rustcodex`");

        if !files.is_empty() {
            s!("Heuristically determined source files:");
        }
        for (name, code) in files.iter().map(|file| (file.name(), file.code())) {
            s!("");
            s!("SOURCE FILE: {name}");
            for line in code.lines() {
                s!("{line}");
            }
        }

        Ok(())
    }
}

/// Reference to in-memory program data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data<'s> {
    payload: &'s [u8],
    sources: &'s [SourceFile],
}

impl<'s> Data<'s> {
    pub const fn new(payload: &'s [u8], sources: &'s [SourceFile]) -> Self {
        Self { payload, sources }
    }
}

/// Template container
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Template<'d, T> {
    data: Data<'d>,
    ctrl: T,
}

impl<'d> Template<'d, ()> {
    pub const fn new(data: Data<'d>) -> Self {
        Self { data, ctrl: () }
    }
}

impl<'d, T: Copy /* `Copy` prevents `Drop` */> Template<'d, T> {
    /// Change controller of the template
    pub const fn transform<U>(self, new: U) -> Template<'d, U> {
        // `Copy` prevents data loss
        Template {
            data: self.data,
            ctrl: new,
        }
    }
}

impl<'d, T: 'd> Template<'d, T>
where
    Self: Display,
{
    /// Dispatch the template dynamically
    pub fn erase(self) -> Box<dyn Display + 'd> {
        Box::new(self)
    }
}

// Import language and template definitions generated in `build.rs`, which also
// sets `GENERATED` location, which (due to `std` macro limitations) must be UTF-8.
// In order not to deal with path separators and stuff, the build sets the correct
// output path. Keep in sync!
include!(env!("GENERATED"));
