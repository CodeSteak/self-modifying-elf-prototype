#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate microwiki_derive;
extern crate ipc;
extern crate nix;
extern crate notify;
extern crate plugin;

use std::sync::*;

use ipc::*;

use plugin::interface::QueryOperation;
pub use runner::PluginInfo;

pub mod core;
pub mod prelude;
pub mod runner;
pub mod util;

lazy_static! {
    pub static ref ROUTING_TABLE: Arc<Mutex<Router<PluginInfo>>> = { Default::default() };
}

lazy_static! {
    pub static ref GLOBAL_STATE: Arc<RwLock<prelude::State>> = {
        let state = core::file::DATA
            .lock()
            .unwrap()
            .read()
            .expect("Unable to read()");

        Arc::new(RwLock::new(state))
    };
}

fn main() {
    if let Ok(dir) = std::env::var("OVERLAY") {
        overlay_main(dir);
    } else {
        normal_main();
    }
}

fn normal_main() {
    let cmd = std::env::args().nth(1).expect("1 Argument needed");

    let q = QueryOperation::And(vec![
        QueryOperation::ByTag(prelude::Tag::new("command", cmd.as_str())),
        QueryOperation::ByTag(prelude::Tag::new("type", "ELF")),
    ]);
    let entities = crate::core::entry::query((q,)).unwrap();
    let ent = entities.get(0).expect("Command Not Found!");

    let args = std::env::args().skip(1).collect::<Vec<String>>();
    runner::start_plugin_by_entry(ent, &args).unwrap().wait();
}

fn overlay_main(dir: String) {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::*;

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).unwrap();
    watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

    let cmd = std::env::args().nth(1).expect("1 Argument needed");

    extern "C" fn die_on_signal(_signal: i32) {
        std::process::exit(0);
    }

    loop {
        match util::directory_overlay::apply(&mut *(GLOBAL_STATE.write().unwrap()), &dir) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        };

        let q = QueryOperation::And(vec![
            QueryOperation::ByTag(prelude::Tag::new("command", cmd.as_str())),
            QueryOperation::ByTag(prelude::Tag::new("type", "ELF")),
        ]);

        let entities = crate::core::entry::query((q,)).unwrap();
        let ent = entities.get(0).expect("Command Not Found!");

        let args = std::env::args().skip(1).collect::<Vec<String>>();

        let pid = runner::start_plugin_by_entry(ent, &args).unwrap();

        use nix::sys::signal::*;
        let handler = SigHandler::Handler(die_on_signal);
        unsafe { signal(Signal::SIGCHLD, handler) }.unwrap();

        while let Ok(_) = rx.try_recv() { /* SKIP */ }
        rx.recv().unwrap();

        std::thread::sleep(Duration::from_millis(25));

        unsafe { signal(Signal::SIGCHLD, SigHandler::SigDfl) }.unwrap();
        println!("Restarting...");
        pid.kill();
    }
}
