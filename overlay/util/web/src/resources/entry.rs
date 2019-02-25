use crate::util::*;
use serde::Deserialize;
use std::collections::BTreeSet;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("entry");
    r.method(http::Method::GET).f(entry);
    r.method(http::Method::DELETE).f(entry_delete)
}

pub(crate) fn register_upload(r: &mut Resource<AppState>) {
    r.name("entry_upload");
    r.method(http::Method::POST)
        .with_config(entry_upload, |((cfg, _),)| {
            cfg.limit(16 * 1024);
        });
}

fn entry_delete(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let ctx = &req.state().ctx;

        let name: String = req.match_info().get("name").and_then(url_decode)?;

        let b = core::entry::write(
            &ctx,
            &WriteOperation::Entry {
                old: Some(name.clone()),
                new: None,
            },
        );

        if b {
            Some(HttpResponse::Ok().json(b))
        } else {
            Some(HttpResponse::NotFound().json(b))
        }
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

fn entry(req: &HttpRequest<AppState>) -> HttpResponse {
    entry_html(req)
}

fn entry_html(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let ctx = &req.state().ctx;
        let p: String = req.match_info().get("name").and_then(url_decode)?;

        let item: Entry = core::entry::read(&ctx, &p)?;

        let get_data /*: FnOnce() -> Option<Vec<u8>>*/ = || {
            let vec : Vec<u8> =core::hash::read(&ctx,&item.data)?.into();
            Some(vec)
        };

        let typ = item
            .tags
            .iter()
            .find(|t| t.name == tag_names::types::TAG)
            .and_then(|t| t.value.as_ref())
            .map(|s| s.as_str());

        match typ {
            Some("text") | Some("txt") => {
                let data = String::from_utf8(get_data()?).ok()?;

                Some(
                    HttpResponse::Ok()
                        .content_type("text/plain; charset=utf-8")
                        .body(html_encode(data.as_str())),
                )
            }
            Some("html") => {
                let data = get_data()?;

                Some(
                    HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(data),
                )
            }
            Some("js") => {
                let data = get_data()?;

                Some(
                    HttpResponse::Ok()
                        .content_type("text/javascript; charset=utf-8")
                        .body(data),
                )
            }
            Some("css") => {
                let data = get_data()?;

                Some(
                    HttpResponse::Ok()
                        .content_type("text/css; charset=utf-8")
                        .body(data),
                )
            }
            Some("mp3") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".mp3"])
                    .unwrap();

                Some(
                    HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(format!("<audio src='{}' controls='controls'></audio>", url)),
                )
            }
            Some("m4a") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".m4a"])
                    .unwrap();

                Some(
                    HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(format!("<audio src='{}' controls='controls'></audio>", url)),
                )
            }
            Some("pdf") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".pdf"])
                    .unwrap();

                Some(
                    HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(format!("<iframe src='{}' frameborder='0'></iframe>", url)),
                )
            }
            _ => Some(
                HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body("<i>Unknown Data</i>"),
            ),
        }
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

#[derive(Deserialize)]
struct EntryUpload {
    old_name: Option<String>,
    name: String,
    data: HashRef,
    tags: BTreeSet<(String, Option<String>)>,
}

fn entry_upload((data, req): (Json<EntryUpload>, HttpRequest<AppState>)) -> HttpResponse {
    let ctx = &req.state().ctx;

    let b = core::entry::write(
        &ctx,
        &WriteOperation::Entry {
            old: data.old_name.clone(),
            new: Some(Entry {
                name: data.name.clone(),
                data: data.data.clone(),
                tags: data
                    .tags
                    .iter()
                    .map(|(k, v)| {
                        if let Some(val) = v {
                            Tag::new(k, (*val).as_str())
                        } else {
                            Tag::new(k, None)
                        }
                    })
                    .collect(),
            }),
        },
    );

    if b {
        HttpResponse::Ok().json(b)
    } else {
        HttpResponse::BadRequest().json(b)
    }
}
