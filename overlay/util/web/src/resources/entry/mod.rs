use crate::util::*;
use serde::Deserialize;
use std::collections::BTreeSet;

mod render;
mod templates;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("entry");
    r.method(http::Method::GET).f(|req| {
        if req.query().contains_key("edit") {
            entry_show_edit(req)
        } else {
            entry_show(req)
        }
    });
}

pub(crate) fn register_root(r: &mut Resource<AppState>) {
    r.name("entry_root");
    r.method(http::Method::GET).with(entry_get);
    r.method(http::Method::DELETE).with(entry_delete);
    r.method(http::Method::PUT).with(entry_upload);
    r.method(http::Method::POST).with(entry_upload_or_delete);
}

fn entry_show(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let ctx = &req.state().ctx;
        let p: String = req.match_info().get("name").and_then(url_decode)?;

        let item: Entry = core::entry::read(&ctx, &p)?;
        let content = render::render_entry(req, &item);
        Some(templates::main(req, (item, content)))
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

fn entry_show_edit(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let ctx = &req.state().ctx;
        let p: String = req.match_info().get("name").and_then(url_decode)?;

        let item: Entry = core::entry::read(&ctx, &p)?;

        let data: Vec<u8> = core::hash::read(&ctx, &item.data)?.into();
        let data: String = String::from_utf8(data).ok()?;

        let tags = item
            .tags
            .iter()
            .map(|tag| {
                if let Some(ref value) = tag.value {
                    format!("{}={}", tag.name, value)
                } else {
                    format!("{}", tag.name)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let cont = format!(
            r#"
            <section>
                <h1> Update </h1>
                <form method="POST" action="{url}?m=put">
                    <input type="hidden" name="old_name" value="{name}"/>

                    <p>
                    <label for="name">Title</label><br/>
                    <input type="text" name="name" value="{name}"/>
                    </p>

                    <p>
                    <label for="tags">Tags</label><br/>
                    <textarea rows="6" cols="60" name="tags">{tags}</textarea>
                    </p>

                    <p>
                    <label for="data">Content</label><br/>
                    <textarea rows="40" cols="60" name="data">{data}</textarea>
                    </p>

                    <p>
                    <input type="submit" value="Update">
                    </p>
                </form>
                <form method="POST" action="{url}?m=delete">
                    <input type="hidden" name="old_name" value="{name}"/>
                    <input type="submit" value="Delete {name}">
                </from>
            </section>
        "#,
            name = html_encode(&p),
            data = html_encode(&data),
            tags = tags,
            url = req.url_for("entry_root", &[] as &[&str]).unwrap(),
        );

        Some(templates::main(req, cont))
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

fn entry_get(req: HttpRequest<AppState>) -> HttpResponse {
    let ctx = &req.state().ctx;

    if let Some(q) = req.query().get("q") {
        let items: Vec<(Entry, String)> = core::entry::query(
            ctx,
            &QueryOperation::Or(vec![
                QueryOperation::HasInName(q.clone()),
                QueryOperation::HasInTagName(q.clone()),
            ]),
        )
        .into_iter()
        .flat_map(|item| {
            let content = render::render_entry(&req, &item);
            Some((item, content))
        })
        .collect();

        return templates::main(&req, &items[..]);
    }

    // Create New
    let cont = format!(
        r#"
            <section>
                <h1> Create New </h1>
                <form method="POST" action="{url}?m=put">
                    <p>
                    <label for="name">Title</label><br/>
                    <input type="text" name="name" value=""/>
                    </p>

                    <p>
                    <label for="tags">Tags</label><br/>
                    <textarea rows="6" cols="60" name="tags">type=text</textarea>
                    </p>

                    <p>
                    <label for="data">Content</label><br/>
                    <textarea rows="40" cols="60" name="data"></textarea>
                    </p>

                    <p>
                    <input type="submit" value="Create New">
                    </p>
                </form>
            </section>
        "#,
        url = req.url_for("entry_root", &[] as &[&str]).unwrap(),
    );

    return templates::main(&req, cont);
}

#[derive(Deserialize)]
struct EntryUploadOrDelete {
    old_name: Option<String>,
    name: Option<String>,
    data: Option<String>,
    tags: Option<String>,
}

fn entry_upload_or_delete(
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
                return entry_upload((
                    Form(EntryUpload {
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
                return entry_delete((
                    Form(EntryDelete {
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

#[derive(Deserialize)]
struct EntryUpload {
    old_name: Option<String>,
    name: String,
    data: String,
    tags: String,
}

fn entry_upload((data, req): (Form<EntryUpload>, HttpRequest<AppState>)) -> HttpResponse {
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

#[derive(Deserialize)]
struct EntryDelete {
    old_name: String,
}
fn entry_delete((data, req): (Form<EntryDelete>, HttpRequest<AppState>)) -> HttpResponse {
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
