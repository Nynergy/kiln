use colored::Colorize;
use std::{
    fmt,
    path::PathBuf,
};

use crate::types::id3::{
    TagPair,
    TagSet,
};

pub struct KilnError {
    pub kind: KilnErrorKind,
    pub message: String,
}

impl KilnError {
    pub fn new(kind: KilnErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl fmt::Display for KilnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KilnError: {:?} => {}", self.kind, self.message)
    }
}

#[derive(Debug)]
pub enum KilnErrorKind {
    File,
    Glob,
    ID3,
    Image,
    Parse,
}

pub type KilnResult<T> = Result<T, KilnError>;

#[derive(Debug)]
pub struct Section {
    pub header: String,
    pub tag_set: TagSet,
}

pub struct FileDiff {
    pub filepath: PathBuf,
    pub diffs: Vec<Diff>,
}

impl FileDiff {
    pub fn from(header: String) -> Self {
        Self {
            filepath: PathBuf::from(header),
            diffs: Vec::new(),
        }
    }
}

impl fmt::Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let diffs = self.diffs.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let path_string = self.filepath.clone().into_os_string().into_string().unwrap();

        write!(f, "[{}]\n{}", path_string, diffs)
    }
}

pub enum Diff {
    Add(TagPair),
    Delete(TagPair),
    Modify(TagPair, TagPair),
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Diff::Add(tag) => write!(f, "{}", format!("{} {:?}: {}", "A".bold(), tag.id, tag.val).green()),
            Diff::Delete(tag) => write!(f, "{}", format!("{} {:?}: {}", "D".bold(), tag.id, tag.val).red()),
            Diff::Modify(old, new) => write!(f, "{}", format!("{} {:?}: {} -> {:?}: {}", "M".bold(), old.id, old.val, new.id, new.val).yellow()),
        }
    }
}
