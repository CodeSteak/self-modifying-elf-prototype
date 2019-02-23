use crate::prelude::*;

use std::sync::*;

#[service(("core","hash","list"))]
fn list(_: Vec<()>) -> Option<Vec<HashRef>> {
    let state = GLOBAL_STATE.read().unwrap();
    Some(state.data.keys().map(HashRef::clone).collect())
}

#[service(("core","hash","read", NULL))]
pub fn read((h,): (HashRef,)) -> Option<Arc<ByteBuf>> {
    let state = GLOBAL_STATE.read().unwrap();
    let h = state.data.get(&h)?;
    match h {
        DataSource::Memory(data) => Some(data.clone()),
        _ => unimplemented!(),
    }
}
