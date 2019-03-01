use crate::util::*;

pub(crate) fn get_entry(req: &HttpRequest<AppState>) -> HttpResponse {
    if req.query().contains_key("edit") {
        entry_show_edit(req)
    } else {
        entry_show(req)
    }
}

fn entry_show(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let ctx = &req.state().ctx;
        let p: String = req.match_info().get("name").and_then(url_decode)?;

        let item: Entry = core::entry::read(&ctx, &p)?;
        let content = super::render::render_entry(req, &item);
        Some(super::templates::main(req, (item, content)))
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

        Some(super::templates::main(req, cont))
    })()
    .unwrap_or_else(|| HttpResponse::NotFound().body("Not Found!"))
}

pub(crate) fn entry_search(req: HttpRequest<AppState>) -> HttpResponse {
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
            let content = super::render::render_entry(&req, &item);
            Some((item, content))
        })
        .collect();

        return super::templates::main(&req, &items[..]);
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

    return super::templates::main(&req, cont);
}
