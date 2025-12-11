use core::{borrow::Borrow, fmt};

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PathBuf(Vec<Box<str>>);

impl PathBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<P: ToString>(&mut self, part: P) {
        self.0.push(part.to_string().into_boxed_str())
    }

    pub fn parent(&self) -> Option<&Path> {
        if self.0.is_empty() {
            return None;
        }

        let slice = &self.0[..self.0.len() - 1];

        Some(Path::from_slice(slice))
    }

    pub fn base(&self) -> Option<&str> {
        if self.0.is_empty() {
            return None;
        }

        Some(&*self.0[self.0.len() - 1])
    }
}

impl From<String> for PathBuf {
    fn from(path: String) -> Self {
        let trimmed = path.trim_start_matches("/");
        let parts = trimmed
            .split("/")
            .map(|part| part.to_owned().into_boxed_str())
            .collect();

        Self(parts)
    }
}

pub struct Path(pub(crate) [Box<str>]);

impl Path {
    pub fn from_slice(slice: &[Box<str>]) -> &Self {
        // SAFETY: Path is repr(transparent) with [Box<str>] layout
        unsafe { &*(slice as *const [Box<str>] as *const Self) }
    }

    pub fn parent(&self) -> Option<&Self> {
        if self.0.is_empty() {
            return None;
        }

        let slice = &self.0[..self.0.len() - 1];

        Some(Path::from_slice(slice))
    }

    pub fn base(&self) -> Option<&str> {
        if self.0.is_empty() {
            return None;
        }

        Some(&*self.0[self.0.len() - 1])
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        Path::from_slice(&self.0)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        write!(f, "{}", self.0[0])?;

        for el in &self.0[1..] {
            write!(f, "/{}", el)?;
        }

        Ok(())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        write!(f, "{}", self.0[0])?;

        for el in &self.0[1..] {
            write!(f, "/{}", el)?;
        }

        Ok(())
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        self.as_ref()
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;

    fn to_owned(&self) -> Self::Owned {
        PathBuf(self.0.to_vec())
    }
}

impl Serialize for PathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(Self::from(string))
    }
}
