use crate::util::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct EntryUploadOrDelete {
    pub(crate) old_name: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate)  data: Option<String>,
    pub(crate) tags: Option<String>,
}

pub(crate) fn entry_upload_or_delete(
    (data, req): (Form<EntryUploadOrDelete>, HttpRequest<AppState>),
) -> HttpResponse {
    let m_value = req.query().get("m").cloned();
    match m_value.as_ref().map(|s| s.as_str()) {
        Some("put") => {
            if let EntryUploadOrDelete {
                ref old_name,
                name: Some(ref name),
                data: Some(ref data),
                tags: Some(ref tags),
            } = *data
            {
                return super::put::entry_upload((
                    Form(super::put::EntryUpload {
                        old_name: old_name.clone(),
                        name: name.clone(),
                        data: data.clone(),
                        tags: tags.clone(),
                    }),
                    req,
                ));
            }
        }
        Some("delete") => {
            if let EntryUploadOrDelete {
                old_name: Some(ref old_name),
                name: None,
                data: None,
                tags: None,
            } = *data
            {
                return super::delete::entry_delete((
                    Form(super::delete::EntryDelete {
                        old_name: old_name.clone(),
                    }),
                    req,
                ));
            }
        }
        _ => (),
    }

    HttpResponse::BadRequest().body("Invalid Format!")
}


