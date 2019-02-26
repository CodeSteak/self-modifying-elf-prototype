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
            &WriteEntry {
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

fn html_find_all(ctx: &Channel<()>, tag: &str) -> String {
    core::entry::query(ctx, &QueryOperation::ByTagName(tag.into()))
        .into_iter()
        .flat_map(|e| core::hash::read(&ctx, &e.data))
        .flat_map(|data| String::from_utf8(data.into()).ok())
        .collect::<Vec<_>>()
        .join("\n\t\t")
}

fn render_into_template(
    ctx: &Channel<()>,
    ty: Option<String>,
    entry: &Entry,
    content: String,
) -> Option<HttpResponse> {
    let styles = core::entry::query(ctx, &QueryOperation::ByTagName("web-ui/style".into()))
        .into_iter()
        .map(|e| format!(r#"<link rel="stylesheet" href="/hash/{}.css"/>"#, e.data))
        .collect::<Vec<_>>()
        .join("\n\t\t");

    let scripts = core::entry::query(ctx, &QueryOperation::ByTagName("web-ui/script".into()))
        .into_iter()
        .map(|e| format!(r#"<script src="/hash/{}.js"></script>"#, e.data))
        .collect::<Vec<_>>()
        .join("\n\t\t");

    let html_header = html_find_all(ctx, "web-ui/html-header");

    let html_footer = html_find_all(ctx, "web-ui/html-footer");

    let html_metadata = html_find_all(ctx, "web-ui/html-metadata");

    let html_section_begin = html_find_all(ctx, "web-ui/html-section-begin");

    let html_section_middle = html_find_all(ctx, "web-ui/html-section-middle");

    let html_section_end = html_find_all(ctx, "web-ui/htm-section-end");

    let body = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="UTF-8"/>
                <title>{entry_name}</title>
                {styles}
                {html_metadata}
            </head>
            <body>
                {html_header}
                <main>
                    <section class="{typ}" id="{entry_name}">
                        {html_section_begin}
                        <h1>{entry_name}</h1>
                        {html_section_middle}
                        {content}
                        {html_section_end}
                    <section>
                </main>
                {scripts}
                {html_footer}
            <body>
        </html>
    "#,
        entry_name = html_encode(&entry.name),
        styles = styles,
        scripts = scripts,
        content = content,
        html_footer = html_footer,
        html_header = html_header,
        html_metadata = html_metadata,
        html_section_begin = html_section_begin,
        html_section_middle = html_section_middle,
        html_section_end = html_section_end,
        typ = html_encode(&ty.unwrap_or_else(|| String::new()))
    );

    let res = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body);

    Some(res)
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

        let typ_owned = typ.map(|s| s.to_string());

        match typ {
            Some("text") | Some("txt") => {
                let data = String::from_utf8(get_data()?).ok()?;
                render_into_template(ctx, typ_owned, &item, html_encode(data.as_str()))
            }
            Some("html") => {
                let data = String::from_utf8(get_data()?).ok()?;

                render_into_template(ctx, typ_owned, &item, data)
            }
            Some("js") | Some("css") | Some("rs") => {
                let data = String::from_utf8(get_data()?).ok()?;

                let highlit = syntax_highlight_html(&data, typ.as_ref().unwrap())
                    .unwrap_or_else(|| html_encode(&data));

                render_into_template(
                    ctx,
                    typ_owned,
                    &item,
                    format!("<pre><code>{}</code></pre>", highlit),
                )
            }
            Some("mp3") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".mp3"])
                    .unwrap();

                render_into_template(
                    ctx,
                    typ_owned,
                    &item,
                    format!("<audio src='{}' controls='controls'></audio>", url),
                )
            }
            Some("m4a") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".m4a"])
                    .unwrap();

                render_into_template(
                    ctx,
                    typ_owned,
                    &item,
                    format!("<audio src='{}' controls='controls'></audio>", url),
                )
            }
            Some("pdf") => {
                let url = req
                    .url_for("hash.ext", &[&format!("{}", item.data), ".pdf"])
                    .unwrap();

                render_into_template(
                    ctx,
                    typ_owned,
                    &item,
                    format!("<iframe src='{}'  frameborder='0'></iframe>", url),
                )
            }
            _ => {
                let url = req
                    .url_for(
                        "hash.ext",
                        &[&format!("{}", item.data), &format!("/{}", item.name)],
                    )
                    .unwrap();

                render_into_template(
                    ctx,
                    typ_owned,
                    &item,
                    format!(
                        r#"
                    <a href='{url}'> {name} </a>
                "#,
                        url = url,
                        name = &item.name
                    ),
                )
            }
        }
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

fn syntax_highlight_html(data: &str, ty: &str) -> Option<String> {
    use syntect::highlighting::{ThemeSet};
    use syntect::html::highlighted_html_for_string;
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut html = highlighted_html_for_string(
        data,
        &ss,
        ss.find_syntax_by_extension(ty)?,
        ts.themes.iter().nth(1).unwrap().1,
    )
    .lines()
    .enumerate()
    .skip(1)
    .map(|(n, l)| format!("<span class='line-number'>{:4}   </span>{}", n, l))
    .collect::<Vec<_>>()
    .join("\n");
    html.truncate(html.len() - "</pre>".len());
    Some(html)
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
        &WriteEntry {
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
