use crate::prelude::*;

#[service(("core","entry","list"))]
pub fn list(_: Vec<()>) -> Option<Vec<Entry>> {
    query((QueryOperation::And(vec![]),))
}

#[service(("core","entry","query", NULL))]
pub fn query((q,): (QueryOperation,)) -> Option<Vec<Entry>> {
    let state = GLOBAL_STATE.read().unwrap();
    q.apply(&*state).into()
}

#[service(("core","entry","read", NULL))]
pub fn read((s,): (String,)) -> Option<Entry> {
    let state = GLOBAL_STATE.read().unwrap();

    state.entries.get(&s).cloned()
}

#[service(("core","entry","write", NULL))]
pub fn write((op,): (WriteOperation,)) -> Option<bool> {
    let mut state = GLOBAL_STATE.write().unwrap();

    let change = op.clone().apply(&mut *state);
    if change {
        crate::core::file::write((op.clone(),))?;
    }

    Some(change)
}
