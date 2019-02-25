use crate::prelude::*;

#[service("core", "entry", "list")]
pub fn list(_ctx: ()) -> Vec<Entry> {
    query((), QueryOperation::And(vec![]))
}

#[service("core", "entry", "query")]
pub fn query(_ctx: (), q: QueryOperation) -> Vec<Entry> {
    let state = GLOBAL_STATE.read().unwrap();
    q.apply(&*state).into()
}

#[service("core", "entry", "read")]
pub fn read(_ctx: (), s: String) -> Option<Entry> {
    let state = GLOBAL_STATE.read().unwrap();

    state.entries.get(&s).cloned()
}

#[service("core", "entry", "write")]
pub fn write(_ctx: (), op: WriteOperation) -> bool {
    let mut state = GLOBAL_STATE.write().unwrap();

    let change = op.clone().apply(&mut *state);
    if change {
        if !crate::core::file::write((), op.clone()) {
            panic!("Failed writing data!");
        }
    }

    change
}
