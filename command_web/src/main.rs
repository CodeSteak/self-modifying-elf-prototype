extern crate ipc;
extern crate plugin;

extern crate actix_web;
extern crate serde;
extern crate serde_json;
extern crate syntect;

mod resources;
mod util;

use util::*;

fn main() {
    const ADDR : &'static str = "[::1]:8080";

    let ch: Channel = Channel::new_from_env(());

    let sys = actix::System::new("sys");

    let state = AppState { ctx: ch };

    server::new(move || {
        App::with_state(state.clone())
            .resource("/", resources::index)
            .resource("/hash/", resources::hash_upload)
            .resource("/hash/{hash:[a-fA-F0-9]{64}}", resources::hash)
            .resource(
                "/hash/{hash:[a-fA-F0-9]{64}}{ext:[\\./].*}",
                resources::hash_ext,
            )
            .resource("/entry/{name:.+}", resources::entry)
            .resource("/entry/", resources::entry_root)
            .default_resource(|r| {
                r.f(|_| HttpResponse::NotFound().body("<h1>404</h1><h3>Not found!</h3>"))
            })
    })
    .bind(ADDR)
    .unwrap()
    .start();

    println!("Listening on {}", ADDR);

    let _ = sys.run();
}
