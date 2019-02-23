use crate::util::*;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("entry");
    r.f(entry);
}

fn entry(req: &HttpRequest<AppState>) -> HttpResponse {
    entry_html(req)
}

fn entry_html(req: &HttpRequest<AppState>) -> HttpResponse {
    (|| {
        let mut ctx = req.state().ctx.clone();
        let p: String = req.match_info().get("name").and_then(url_decode)?;
        let item: Entry = ctx
            .call::<_, Vec<Entry>>(&(
                "core",
                "entry",
                "query",
                QueryOperation::ByName(p.to_string()),
            ))?
            .into_iter()
            .next()?;

        let mut get_data /*: FnOnce() -> Option<Vec<u8>>*/ = || {
            let vec : Vec<u8> = (ctx.call::<_,ByteBuf>(&("core","hash","read",&item.data)) as Option<ByteBuf>)?.into();
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
            },
            Some("js") => {
                let data = get_data()?;

                Some(
                    HttpResponse::Ok()
                        .content_type("text/javascript; charset=utf-8")
                        .body(data),
                )
            },
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
