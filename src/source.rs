use std::cmp::Ordering;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io::Error;
use std::iter::Chain;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub name: String,
    pub code: String,
}

impl SourceFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        read_to_string(path).map(|code| SourceFile {
            name: path.display().to_string(),
            code,
        })
    }
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

pub trait Source {
    type Container: IntoIterator<Item = SourceFile>;
    fn sources(self) -> Result<Self::Container, Error>;
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
    fn erase(self) -> Erased
    where
        Self: Sized + 'static,
    {
        Erased::sourced(self)
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
    #[rustfmt::skip]
    type Container = Chain<
        <F::Container as IntoIterator>::IntoIter,
        <S::Container as IntoIterator>::IntoIter
    >;
    fn sources(self) -> Result<Self::Container, Error> {
        Ok(self
            .first
            .sources()?
            .into_iter()
            .chain(self.second.sources()?))
    }
}

pub type ErasedIter = Box<dyn Iterator<Item = SourceFile>>;

pub struct Erased {
    source: Box<dyn FnOnce() -> Result<ErasedIter, Error>>,
}

impl Erased {
    fn sourced<S: Source + 'static>(source: S) -> Self {
        let convert = |typed: S::Container| Box::new(typed.into_iter()) as ErasedIter;
        let source = Box::new(move || source.sources().map(convert));
        Self { source }
    }
}

impl Source for Erased {
    type Container = ErasedIter;
    fn sources(self) -> Result<Self::Container, Error> {
        (self.source)()
    }
}

impl<T, P> Source for T
where
    P: AsRef<Path>,
    T: IntoIterator<Item = P>,
{
    type Container = Vec<SourceFile>;
    fn sources(self) -> Result<Self::Container, Error> {
        self.into_iter().map(SourceFile::load).collect()
    }
}

//impl Source for Box<dyn Source> {
//    fn sources(&self) -> Result<Box<dyn Iterator<Item = SourceFile>>, Error> {
//        (**self).sources()
//    }
//}

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
            buf.push(SourceFile::load(path)?)
        }
    }
    Ok(buf)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rust;

impl Source for Rust {
    type Container = Vec<SourceFile>;
    fn sources(self) -> Result<Self::Container, Error> {
        let mut files = Vec::new();
        expander(&mut files, "src".as_ref())?.sort_unstable_by(|first, second| {
            match (first.name.as_str(), second.name.as_str()) {
                (first, second) if first == second => Ordering::Equal,
                ("src/main.rs", _) => Ordering::Less,
                (_, "src/main.rs") => Ordering::Greater,
                ("src/lib.rs", _) => Ordering::Less,
                (_, "src/lib.rs") => Ordering::Greater,
                _ => first.cmp(second),
            }
        });
        Ok(files)
    }
}
