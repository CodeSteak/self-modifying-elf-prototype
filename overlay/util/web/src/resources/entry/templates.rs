use crate::util::*;

pub(crate) trait MainContent {
    fn render(self, req: &HttpRequest<AppState>) -> String;
}

impl MainContent for String {
    fn render(self, _: &HttpRequest<AppState>) -> String {
        self
    }
}
impl MainContent for (Entry, String) {
    fn render(self, req: &HttpRequest<AppState>) -> String {
        (&[self][..]).render(req)
    }
}

impl MainContent for &[(Entry, String)] {
    fn render(self, req: &HttpRequest<AppState>) -> String {
        let sections = self
            .iter()
            .map(|(entry, content)| {
                let section = format!(
                    r#"
            <section class="{typ}" id="{entry_name}">
                        <h1>{entry_name}</h1>
                        <a class="edit" href="{edit_url}?edit=true">[edit]</a>
                        {content}
            </section>
        "#,
                    entry_name = html_encode(&entry.name),
                    typ = entry
                        .tags
                        .iter()
                        .filter(|tag| tag.name == tag_names::types::TAG)
                        .map(|tag| html_encode(
                            tag.value.as_ref().map(|v| v.as_str()).unwrap_or("")
                        ))
                        .collect::<Vec<_>>()
                        .join(" "),
                    edit_url = req.url_for("entry", &[&entry.name],).unwrap(),
                    content = content
                );

                section
            })
            .collect::<Vec<_>>()
            .join("\n\t\t");

        sections
    }
}

pub(crate) fn main<S: MainContent + Sized>(req: &HttpRequest<AppState>, items: S) -> HttpResponse {
    let ctx = &req.state().ctx;

    let styles = core::entry::query(ctx, &QueryOperation::ByTagName("web-ui/style".into()))
        .into_iter()
        .map(|e| {
            format!(
                r#"<link rel="stylesheet" href="/hash/{}/{}.css"/>"#,
                e.data,
                url_encode(&e.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\t\t");

    let scripts = core::entry::query(ctx, &QueryOperation::ByTagName("web-ui/script".into()))
        .into_iter()
        .map(|e| {
            format!(
                r#"<script src="/hash/{}/{}.js"></script>"#,
                e.data,
                url_encode(&e.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\t\t");

    let html_header = get_entries_with_tag(ctx, "web-ui/html-header");

    let html_footer = get_entries_with_tag(ctx, "web-ui/html-footer");

    let html_metadata = get_entries_with_tag(ctx, "web-ui/html-metadata");

    let body = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="UTF-8"/>
                {styles}
                {html_metadata}
            </head>
            <body>
                {html_header}
                <main>
                    {sections}
                </main>
                <nav>
                    <p>
                    <a href="{url_new}">Create new</a>
                    </p>
                    <p>
                    <form action="/entry/" method="get">
                        <input type="text" name="q"/>
                        <input type="submit" value="Search">
                    </form>
                    </p>
                </nav>
                {scripts}
                {html_footer}
            <body>
        </html>
    "#,
        styles = styles,
        scripts = scripts,
        html_footer = html_footer,
        html_header = html_header,
        html_metadata = html_metadata,
        sections = items.render(req),
        url_new = req.url_for("entry_root", &[] as &[&str]).unwrap(),
    );

    let res = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body);

    res
}

fn get_entries_with_tag(ctx: &Channel<()>, tag: &str) -> String {
    core::entry::query(ctx, &QueryOperation::ByTagName(tag.into()))
        .into_iter()
        .flat_map(|e| core::hash::read(&ctx, &e.data))
        .flat_map(|data| String::from_utf8(data.into()).ok())
        .collect::<Vec<_>>()
        .join("\n\t\t")
}
