use crate::util::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct EntryDelete {
    pub(crate) old_name: String,
}

pub(crate) fn entry_delete(
    (data, req): (Form<EntryDelete>, HttpRequest<AppState>),
) -> HttpResponse {
    let b = core::entry::write(
        &req.state().ctx,
        &WriteEntry {
            old: Some(data.old_name.clone()),
            new: None,
        },
    );

    if b {
        let url = req.url_for("index", &[] as &[&str]).unwrap();
        HttpResponse::SeeOther()
            .header("location", url.to_string())
            .body("Deleted")
    } else {
        HttpResponse::NotFound().body("Content to delete not found.")
    }
}
