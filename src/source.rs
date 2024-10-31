use std::cmp::Ordering;
use std::fs::read_dir;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;

use crate::lang::Rust;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    inner: Box<str>,
    split: usize,
}

impl SourceFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut buf = path.display().to_string();
        let split = buf.len();
        buf.try_reserve_exact(file.metadata()?.len() as usize)?;
        file.read_to_string(&mut buf)?;
        Ok(Self {
            inner: buf.into_boxed_str(),
            split,
        })
    }

    pub fn name(&self) -> &str {
        &self.inner[0..self.split]
    }

    pub fn code(&self) -> &str {
        &self.inner[self.split..]
    }
}

impl PartialOrd for SourceFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceFile {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name().cmp(other.name()) {
            Ordering::Equal => self.code().cmp(other.code()),
            ord => ord,
        }
    }
}

pub trait Source {
    fn extend(&self, buf: &mut Vec<SourceFile>) -> Result<(), Error>;
    fn sources(&self) -> Result<Vec<SourceFile>, Error> {
        let mut buf = Vec::new();
        self.extend(&mut buf)?;
        Ok(buf)
    }
    fn merge<O>(self, other: O) -> Merged<Self, O>
    where
        Self: Sized,
        O: Sized + Source,
    {
        Merged {
            first: self,
            second: other,
        }
    }
}

pub struct Merged<F, S>
where
    F: Source,
    S: Source,
{
    first: F,
    second: S,
}

impl<F, S> Source for Merged<F, S>
where
    F: Source,
    S: Source,
{
    fn extend(&self, buf: &mut Vec<SourceFile>) -> Result<(), Error> {
        self.first.extend(buf)?;
        self.second.extend(buf)?;
        Ok(())
    }
}

impl<P> Source for Vec<P>
where
    P: AsRef<Path>,
{
    fn extend(&self, buf: &mut Vec<SourceFile>) -> Result<(), Error> {
        let iter = self.iter().map(SourceFile::load);
        buf.try_reserve(self.len())?;
        for file in iter {
            buf.push(file?);
        }
        Ok(())
    }
}

fn expander<'a, 'b: 'a>(
    buf: &'b mut Vec<SourceFile>,
    path: &Path,
) -> Result<&'a mut Vec<SourceFile>, Error> {
    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            expander(buf, path.as_path())?;
        } else {
            buf.push(SourceFile::load(path)?);
        }
    }
    Ok(buf)
}

impl Source for Rust {
    fn extend(&self, buf: &mut Vec<SourceFile>) -> Result<(), Error> {
        expander(buf, "src".as_ref())?.sort_unstable_by(|first, second| {
            match (first.name(), second.name()) {
                (first, second) if first == second => Ordering::Equal,
                ("src/main.rs", _) => Ordering::Less,
                (_, "src/main.rs") => Ordering::Greater,
                ("src/lib.rs", _) => Ordering::Less,
                (_, "src/lib.rs") => Ordering::Greater,
                _ => first.cmp(second),
            }
        });
        Ok(())
    }
}
