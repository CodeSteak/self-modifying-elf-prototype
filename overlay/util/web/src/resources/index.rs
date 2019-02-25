use crate::util::*;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("index");
    r.f(index);
}

fn index(req: &HttpRequest<AppState>) -> HttpResponse {
    let ctx = &req.state().ctx;

    let res = core::entry::list(&ctx);

    match req
        .headers()
        .get(actix_web::http::header::ACCEPT)
        .and_then(|h| h.to_str().ok())
    {
        Some(h) if h.contains("text/html") => render_html(res, req),
        Some(h) if h.contains("application/json") => render_json(res, req),
        _ => render_html(res, req),
    }
}

fn render_json(data: Vec<Entry>, _req: &HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&data).unwrap())
}

fn render_html(data: Vec<Entry>, req: &HttpRequest<AppState>) -> HttpResponse {
    let mut s = String::new();

    s += "<h1>INDEX</h1>";
    s += "<ul>";
    for item in data {
        s += &format!(
            "<li><a href=\"{}\">{}</a><br/>",
            req.url_for("entry", &[&url_encode(&item.name)]).unwrap(),
            item.name
        );
        s += &format!(
            "<small style='margin-left: 3em;'><a href=\"{}\">{}</a></small>",
            req.url_for(
                "hash.ext",
                &[
                    &format!("{}", item.data),
                    &format!("/{}", url_encode(&item.name))
                ]
            )
            .unwrap(),
            &format!("{}", item.data)
        );
        s += "</li>";
    }
    s += "</ul>";

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(s)
}
