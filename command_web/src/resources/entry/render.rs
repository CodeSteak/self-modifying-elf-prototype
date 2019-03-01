use crate::util::*;

pub(crate) fn render_entry(req: &HttpRequest<AppState>, item: &Entry) -> String {
    let typ = item
        .tags
        .iter()
        .find(|t| t.name == tag_names::types::TAG)
        .and_then(|t| t.value.as_ref())
        .map(|s| s.as_str());

    let ctx = TypeRenderer {
        item,
        req,
        typ: typ.unwrap_or(""),
    };

    ctx.render().unwrap_or_else(|| render_fallback(&ctx))
}

struct TypeRenderer<'a> {
    item: &'a Entry,
    req: &'a HttpRequest<AppState>,
    typ: &'a str,
}

impl<'a> TypeRenderer<'a> {
    fn render(&self) -> Option<String> {
        for (k, v) in TYPE_TABLE.iter() {
            if k == &self.typ.to_lowercase() {
                return v(self);
            }
        }

        return None;
    }
    fn raw(&self) -> Option<Vec<u8>> {
        let ctx = &self.req.state().ctx;
        let vec: Vec<u8> = core::hash::read(&ctx, &self.item.data)?.into();
        Some(vec)
    }

    fn string(&self) -> Option<String> {
        String::from_utf8(self.raw()?).ok()
    }

    fn escaped(&self) -> Option<String> {
        html_encode(&self.string()?).into()
    }

    fn hash_url(&self) -> String {
        self.req
            .url_for("hash.ext", &[&format!("{}", self.item.data), self.typ])
            .unwrap()
            .to_string()
    }
}

static TYPE_TABLE: [(&'static str, fn(&TypeRenderer) -> Option<String>); 30] = [
    // web
    ("css", render_syntax),
    ("html", render_html),
    // more languages
    ("js", render_syntax),
    ("md", render_syntax),
    ("xml", render_syntax),
    ("rb", render_syntax),
    ("ex", render_syntax),
    ("elm", render_syntax),
    ("erl", render_syntax),
    ("nix", render_syntax),
    //text
    ("txt", render_text),
    ("text", render_text),
    //pdf
    ("pdf", render_iframe),
    //audio
    ("wav", render_audio),
    ("mp3", render_audio),
    ("m4a", render_audio),
    ("ogg", render_audio),
    ("flac", render_audio),
    //video
    ("mp4", render_video),
    ("webm", render_video),
    ("mkv", render_video),
    //images
    ("jpg", render_img),
    ("jpeg", render_img),
    ("bmp", render_img),
    ("ico", render_img),
    ("gif", render_img),
    ("png", render_img),
    ("apng", render_img),
    ("svg", render_img),
    ("webp", render_img),
];

fn render_syntax(ctx: &TypeRenderer) -> Option<String> {
    let highlit = highlight_syntax(&ctx.string()?, ctx.typ).or_else(|| ctx.escaped())?;

    Some(format!("<pre><code>{}</code></pre>", highlit))
}

fn render_text(ctx: &TypeRenderer) -> Option<String> {
    format!("<pre>{}</pre>", &ctx.escaped()?).into()
}

fn render_html(ctx: &TypeRenderer) -> Option<String> {
    ctx.string()
}

fn render_audio(ctx: &TypeRenderer) -> Option<String> {
    Some(format!(
        "<audio src='{}' controls='controls'></audio>",
        ctx.hash_url()
    ))
}

fn render_video(ctx: &TypeRenderer) -> Option<String> {
    Some(format!(
        "<video src='{}' controls='controls'></video>",
        ctx.hash_url()
    ))
}

fn render_img(ctx: &TypeRenderer) -> Option<String> {
    Some(format!(
        "<img src='{}' alt='{}'/>",
        ctx.hash_url(),
        html_encode(&ctx.item.name)
    ))
}

fn render_iframe(ctx: &TypeRenderer) -> Option<String> {
    Some(format!(
        "<iframe src='{}' frameborder='0'></iframe>",
        ctx.hash_url()
    ))
}

fn render_fallback(ctx: &TypeRenderer) -> String {
    format!(
        "<a href='{}'>{}</a>",
        ctx.hash_url(),
        html_encode(&ctx.item.name)
    )
}

fn highlight_syntax(data: &str, ty: &str) -> Option<String> {
    use syntect::highlighting::ThemeSet;
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
