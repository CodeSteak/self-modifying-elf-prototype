use crate::prelude::*;

use std::sync::*;

#[service("core", "hash", "list")]
fn list(_ctx: ()) -> Vec<HashRef> {
    let state = GLOBAL_STATE.read().unwrap();
    state.data.keys().map(HashRef::clone).collect()
}

#[service("core", "hash", "read")]
pub fn read(_ctx: (), h: HashRef) -> Option<Arc<ByteBuf>> {
    let state = GLOBAL_STATE.read().unwrap();
    let h = state.data.get(&h)?;
    match h {
        DataSource::Memory(data) => Some(data.clone()),
        _ => unimplemented!(),
    }
}

#[service("core", "hash", "write")]
pub fn write(_ctx: (), op: WriteSmallData) -> bool {
    let op = WriteOperation::SmallData(op);
    let mut state = GLOBAL_STATE.write().unwrap();

    let change = op.clone().apply(&mut *state);
    if change {
        if !crate::core::file::write((), op.clone()) {
            panic!("Failed writing data!");
        }
    }

    change
}
