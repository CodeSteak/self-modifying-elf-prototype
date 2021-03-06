use crate::util::*;

pub(crate) fn register(r: &mut Resource<AppState>) {
    r.name("hash");
    r.f(hash);
}

pub(crate) fn register_secondary(r: &mut Resource<AppState>) {
    r.name("hash.ext");
    r.f(hash);
}

pub(crate) fn register_upload(r: &mut Resource<AppState>) {
    r.name("hash_upload");
    r.f(hash_upload);
}

fn hash(req: &HttpRequest<AppState>) -> Option<HttpResponse> {
    let ctx = &req.state().ctx;
    let hash: HashRef = req.match_info().get("hash")?.parse::<HashRef>().ok()?;

    let data = core::hash::read(&ctx, &hash);

    if let Some(data) = data {
        let vec: Vec<u8> = data.into();
        Some(HttpResponse::Ok().body(vec))
    } else {
        Some(HttpResponse::NotFound().body("No Data Found!"))
    }
}

fn hash_upload(_req: &HttpRequest<AppState>) -> Option<HttpResponse> {
    eprintln!("TODO");
    None
}
