use crate::util::*;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("hash");
    r.f(hash);
}

pub(crate) fn register_secondary(r: &mut Resource<AppState>) {
    r.name("hash.ext");
    r.f(hash);
}

fn hash(req: &HttpRequest<AppState>) -> Option<HttpResponse> {
    let ctx = req.state().ctx.clone();
    let hash: HashRef = req.match_info().get("hash")?.parse::<HashRef>().ok()?;

    let data = ctx.call::<_, ByteBuf>(&("core", "hash", "read", &hash));

    if let Some(data) = data {
        let vec : Vec<u8> = data.into();
        Some(HttpResponse::Ok().body(vec))
    } else {
        Some(HttpResponse::NotFound().body("No Data Found!"))
    }
}
