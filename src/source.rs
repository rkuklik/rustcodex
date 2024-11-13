use std::cmp::Ordering;
use std::fs::metadata;
use std::fs::read_dir;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    inner: Box<str>,
    split: usize,
}

impl SourceFile {
    pub fn load<I, P>(paths: I) -> Result<Vec<Self>, Error>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let mut buf = Vec::new();
        for path in paths {
            Self::extend(&mut buf, path)?
        }
        Ok(buf)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
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

    pub fn read_recursive(path: impl AsRef<Path>) -> Result<Vec<Self>, Error> {
        let mut buf = Vec::new();
        Self::extend(&mut buf, path).map(|()| buf)
    }

    fn extend(buf: &mut Vec<Self>, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        if !metadata(path)?.is_dir() {
            buf.push(Self::read(path)?);
            return Ok(());
        }
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            Self::extend(buf, path)?
        }
        Ok(())
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
