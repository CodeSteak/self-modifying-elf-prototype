#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate microwiki_derive;
extern crate ipc;
extern crate nix;
extern crate plugin;

use std::sync::*;

use ipc::*;

pub use runner::PluginInfo;
use plugin::interface::QueryOperation;

pub mod core;
pub mod prelude;
pub mod runner;
pub mod util;

lazy_static! {
    pub static ref ROUTING_TABLE: Arc<Mutex<Router<PluginInfo>>> = { Default::default() };
}

lazy_static! {
    pub static ref GLOBAL_STATE: Arc<RwLock<prelude::State>> = {
        let mut state = core::file::DATA
                .lock()
                .unwrap()
                .read()
                .expect("Unable to read()");

        if let Ok(dir) = std::env::var("OVERLAY") {
            match util::directory_overlay::apply(&mut state, &dir) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{:?}", e);
                    std::process::exit(1);
                },
            };
        }

        Arc::new(RwLock::new(
            state
        ))
    };
}

fn main() {
    let _ = GLOBAL_STATE.clone();
    let cmd = std::env::args().nth(1).expect("1 Argument needed");

    let q = QueryOperation::And(vec![
        QueryOperation::ByTag(prelude::Tag::new("command",cmd.as_str())),
        QueryOperation::ByTag(prelude::Tag::new(".type","ELF"))
    ]);
    let entities = crate::core::entry::query((q,)).unwrap();
    let ent = entities.get(0).expect("Command Not Found!");

    let args =  std::env::args().skip(1).collect::<Vec<String>>();
    runner::start_plugin_by_entry(ent, &args);
}
