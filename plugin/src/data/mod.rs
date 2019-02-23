use std::collections::*;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub mod hash_ref;
pub use hash_ref::*;

#[derive(Serialize, Deserialize, Debug, Default, PartialOrd, PartialEq, Ord, Eq, Clone)]
pub struct Tag {
    pub name: String,
    pub value: Option<String>,
}

impl Tag {
    pub fn new<'a, V: Into<Option<&'a str>>>(name: &str, value: V) -> Self {
        let value = value.into().map(|s| s.to_owned());
        Tag {
            name: name.to_owned(),
            value,
        }
    }
}

pub mod tag_names {
    pub mod types {
        pub const TAG: &'static str = ".type";

        pub const TEXT: &'static str = "text";
        pub const ELF: &'static str = "ELF";
        pub const AUDIO: &'static str = "audio";
        pub const VIDEO: &'static str = "video";
        pub const PDF: &'static str = "pdf";
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Ord, Eq, Clone)]
pub struct Entry {
    pub name: String,
    pub tags: BTreeSet<Tag>,
    pub data: HashRef,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum DataSource {
    Memory(Arc<serde_bytes::ByteBuf>),
    FilePosition((u64, u64)),
}

#[derive(Clone, Default, Debug)]
pub struct State {
    pub entries: BTreeMap<String, Entry>,
    pub data: HashMap<HashRef, DataSource>,
}
