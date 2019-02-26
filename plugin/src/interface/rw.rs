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
    HasInName(String),
    HasInTagName(String),
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
            QueryOperation::HasInName(pattern) => {
                value.0.to_lowercase().contains(&pattern.to_lowercase())
            }
            QueryOperation::HasInTagName(pattern) => value
                .1
                .tags
                .iter()
                .find(|tag| tag.name.to_lowercase().contains(&pattern.to_lowercase()))
                .is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WriteEntry {
    pub old: Option<String>,
    pub new: Option<Entry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WriteSmallData {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WriteOperation {
    Entry(WriteEntry),
    SmallData(WriteSmallData),
}

impl WriteOperation {
    pub fn apply(self, state: &mut State) -> bool {
        match self {
            WriteOperation::Entry(WriteEntry { old, new }) => {
                // Ensure Hash is stored.
                if let Some(new) = &new {
                    if !state.data.contains_key(&new.data) {
                        return false;
                    }
                }

                // Remove Old
                if let Some(old) = old {
                    state.entries.remove(&old);
                }

                if let Some(new) = new {
                    // Fail if already exits, to prevent
                    // overwriting by accident.
                    if state.entries.contains_key(&new.name) {
                        return false; //Fail
                    }

                    state.entries.insert(new.name.clone(), new);
                }

                true
            }
            WriteOperation::SmallData(WriteSmallData { data }) => {
                let hash_ref = HashRef::from_data(&data);

                if state.data.contains_key(&hash_ref) {
                    return true;
                }
                state
                    .data
                    .insert(hash_ref, DataSource::Memory(Arc::new(data.into())));

                true
            }
        }
    }
}
