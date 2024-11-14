//! Wrappers around `File` and `stdio` to avoid dynamic dispatch

use std::fs::File;
use std::io::stdin;
use std::io::stdout;
use std::io::Error;
use std::io::Read;
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;
use std::path::Path;

pub enum Input {
    Stdio(StdinLock<'static>),
    File(File),
}

impl Input {
    /// Locks `stdin` for reading
    pub fn stdio() -> Self {
        Self::Stdio(stdin().lock())
    }

    /// Opens the file at `path` for reading
    pub fn file(path: impl AsRef<Path>) -> Result<Self, Error> {
        File::options()
            .read(true)
            .write(false)
            .truncate(false)
            .create(false)
            .open(path.as_ref())
            .map(Self::File)
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self {
            Self::Stdio(io) => io.read(buf),
            Self::File(file) => file.read(buf),
        }
    }
}

pub enum Output {
    Stdio(StdoutLock<'static>),
    File(File),
}

impl Output {
    /// Locks `stdout` for writing
    pub fn stdio() -> Self {
        Self::Stdio(stdout().lock())
    }

    /// Opens the file at `path` for writing
    pub fn file(path: impl AsRef<Path>) -> Result<Self, Error> {
        File::options()
            .read(false)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_ref())
            .map(Self::File)
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        match self {
            Self::Stdio(io) => io.write(buf),
            Self::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        match self {
            Self::Stdio(io) => io.flush(),
            Self::File(file) => file.flush(),
        }
    }
}
