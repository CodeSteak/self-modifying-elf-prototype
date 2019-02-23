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

#[service(("core","entry","write", NULL))]
pub fn write((op,): (WriteOperation,)) -> Option<bool> {
    let mut state = GLOBAL_STATE.write().unwrap();
    crate::core::file::write((op.clone(),))?;
    op.apply(&mut *state).into()
}
