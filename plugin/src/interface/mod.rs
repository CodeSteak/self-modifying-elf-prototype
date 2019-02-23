use crate::data::*;
use std::sync::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QueryOperation {
    And(Vec<QueryOperation>),
    Or(Vec<QueryOperation>),
    Not(Box<QueryOperation>),
    ByName(String),
    ByTag(Tag),
    ByTagName(String),
}

impl QueryOperation {
    pub fn apply(&self, state: &State) -> Vec<Entry> {
        let mut result = vec![];

        for (name, entry) in state.entries.iter() {
            if self.matches((name, entry)) {
                result.push(entry.clone())
            }
        }

        return result;
    }

    fn matches(&self, value: (&String, &Entry)) -> bool {
        match self {
            QueryOperation::And(sub_ops) => {
                for op in sub_ops {
                    if !op.matches(value) {
                        return false;
                    }
                }

                true
            }
            QueryOperation::Or(sub_ops) => {
                for op in sub_ops {
                    if op.matches(value) {
                        return true;
                    }
                }

                false
            }
            QueryOperation::Not(inner) => !inner.matches(value),
            QueryOperation::ByName(name) => value.0 == name,
            QueryOperation::ByTag(tag) => value.1.tags.contains(tag),
            QueryOperation::ByTagName(name) => {
                value.1.tags.iter().find(|tag| &tag.name == name).is_some()
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WriteOperation {
    AddEntry {
        name: String,
        data: HashRef,
    },
    UpdateEntry {
        name: String,
        new_name: Option<String>,
        new_data: Option<HashRef>,
    },
    RemoveEntry {
        name: String,
    },
    AddTag {
        name: String,
        tag: Tag,
    },
    RemoveTag {
        name: String,
        tag: Tag,
    },
    AddSmallData {
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

impl WriteOperation {
    pub fn apply(self, state: &mut State) -> bool {
        match self {
            WriteOperation::AddEntry { name, data } => {
                if !state.data.contains_key(&data) {
                    return false;
                }

                state.entries.insert(
                    name.clone(),
                    Entry {
                        name,
                        tags: Default::default(),
                        data,
                    },
                );

                true
            }
            WriteOperation::UpdateEntry {
                name,
                new_name,
                new_data,
            } => {
                if let Some(mut entry) = state.entries.remove(&name) {
                    if let Some(data) = new_data {
                        entry.data = data;
                    }

                    let name = new_name.unwrap_or(name).clone();
                    entry.name = name.clone();
                    state.entries.insert(name, entry);
                    true
                } else {
                    false
                }
            }
            WriteOperation::RemoveEntry { name } => {
                if let Some(_entry) = state.entries.remove(&name) {
                    true
                } else {
                    false
                }
            }
            WriteOperation::AddTag { name, tag } => {
                if let Some(entry) = state.entries.get_mut(&name) {
                    entry.tags.insert(tag)
                } else {
                    false
                }
            }
            WriteOperation::RemoveTag { name, tag } => {
                if let Some(entry) = state.entries.get_mut(&name) {
                    entry.tags.remove(&tag)
                } else {
                    false
                }
            }
            WriteOperation::AddSmallData { data } => {
                let hash_ref = HashRef::from_data(&data);
                if state.data.contains_key(&hash_ref) {
                    return false;
                }

                state
                    .data
                    .insert(hash_ref, DataSource::Memory(Arc::new(data.into())));
                true
            }
        }
    }
}
