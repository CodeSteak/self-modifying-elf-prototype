use crate::util::*;

pub(crate) mod render;
pub(crate) mod templates;

pub(crate) mod get;
pub(crate) mod post;
pub(crate) mod put;
pub(crate) mod delete;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("entry");
    r.method(http::Method::GET).f(get::get_entry);
}

pub(crate) fn register_root(r: &mut Resource<AppState>) {
    r.name("entry_root");
    r.method(http::Method::GET).with(get::entry_search);
    r.method(http::Method::DELETE).with(delete::entry_delete);
    r.method(http::Method::PUT).with(put::entry_upload);
    r.method(http::Method::POST).with(post::entry_upload_or_delete);
}
