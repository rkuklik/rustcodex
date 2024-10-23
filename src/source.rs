use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Error;
use either::Either;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub name: String,
    pub code: String,
}

impl PartialOrd for SourceFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceFile {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name.cmp(&other.name) {
            Ordering::Equal => self.code.cmp(&other.code),
            ord => ord,
        }
    }
}

impl Source for Vec<PathBuf> {
    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, anyhow::Error> {
        let files = self
            .iter()
            .map(|path| {
                Ok::<_, io::Error>(SourceFile {
                    name: path.display().to_string(),
                    code: read_to_string(path.as_path())?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Box::new(files.into_iter()))
    }
}

pub struct MergedSources<F, S>
where
    F: Source,
    S: Source,
{
    pub first: F,
    pub second: S,
}

impl<F, S> Source for MergedSources<F, S>
where
    F: Source,
    S: Source,
{
    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, anyhow::Error> {
        let iter = self.first.sources()?.chain(self.second.sources()?);
        Ok(Box::new(iter))
    }
}

pub trait Source {
    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, anyhow::Error>;
}

impl Source for Box<dyn Source> {
    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, anyhow::Error> {
        (**self).sources()
    }
}

fn expander(path: &Path) -> Result<Vec<SourceFile>, Error> {
    let iter = read_dir(path)
        .map_err(Error::from)
        .with_context(|| format!("`{}` must be accessible", path.display()))?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            Ok::<_, Error>(if entry.file_type()?.is_dir() {
                Either::Left(expander(path.as_path())?)
            } else {
                Either::Right(SourceFile {
                    name: path.display().to_string(),
                    code: read_to_string(path.as_path())?,
                })
            })
        });
    let mut buf = Vec::new();
    for item in iter {
        match item? {
            Either::Left(mut vec) => buf.append(&mut vec),
            Either::Right(source) => buf.push(source),
        }
    }
    Ok(buf)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rust;

impl Source for Rust {
    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, anyhow::Error> {
        let mut files = expander("src".as_ref())?;
        files.sort_unstable_by(
            |first, second| match (first.name.as_str(), second.name.as_str()) {
                ("src/main.rs", _) => Ordering::Less,
                (_, "src/main.rs") => Ordering::Greater,
                ("src/lib.rs", _) => Ordering::Less,
                (_, "src/lib.rs") => Ordering::Greater,
                _ => first.cmp(second),
            },
        );
        Ok(Box::new(files.into_iter()))
    }
}
