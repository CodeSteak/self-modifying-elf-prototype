use crate::util::*;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
pub(crate) struct EntryUpload {
    pub(crate) old_name: Option<String>,
    pub(crate) name: String,
    pub(crate) data: String,
    pub(crate) tags: String,
}
pub(crate) fn entry_upload(
    (data, req): (Form<EntryUpload>, HttpRequest<AppState>),
) -> HttpResponse {
    let content = data.data.clone().into_bytes();
    let hash_ref = HashRef::from_data(&content);

    let _ = core::hash::write(&req.state().ctx, &WriteSmallData { data: content });

    fn parse_tags(cont: &str) -> BTreeSet<Tag> {
        let mut ret: BTreeSet<Tag> = Default::default();
        for line in cont.lines() {
            let mut parts = line.splitn(2, "=");

            match (parts.next(), parts.next()) {
                (Some(key), None) => {
                    ret.insert(Tag::new(key.trim(), None));
                }
                (Some(key), Some(value)) => {
                    ret.insert(Tag::new(key.trim(), value.trim()));
                }
                (None, None) => {
                    //
                }
                _ => unreachable!(),
            }
        }

        ret
    }

    let EntryUpload {
        ref old_name,
        ref name,
        ref tags,
        ..
    } = *data;

    let b = core::entry::write(
        &req.state().ctx,
        &WriteEntry {
            old: old_name.clone(),
            new: Some(Entry {
                name: name.clone(),
                data: hash_ref.clone(),
                tags: parse_tags(tags),
            }),
        },
    );

    if b {
        let url = req.url_for("entry", &[name.as_str()]).unwrap();

        HttpResponse::SeeOther()
            .header("location", url.to_string())
            .body("Updated")
    } else {
        HttpResponse::NotFound().body("Failed.")
    }
}
