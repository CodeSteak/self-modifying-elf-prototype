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
    Entry {
        old: Option<String>,
        new: Option<Entry>,
    },
    SmallData {
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

impl WriteOperation {
    pub fn apply(self, state: &mut State) -> bool {
        match self {
            WriteOperation::Entry { old, new } => {
                if let Some(new) = &new {
                    if !state.data.contains_key(&new.data) {
                        return false;
                    }
                }

                if let Some(old) = old {
                    state.entries.remove(&old);
                }

                if let Some(new) = new {
                    state.entries.insert(new.name.clone(), new);
                }

                true
            }
            WriteOperation::SmallData { data } => {
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
