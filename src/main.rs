#[macro_use]
extern crate lazy_static;

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

#[context]
#[derive(Clone, Default)]
pub struct Context {
    pub plugin_info: Option<PluginInfo>,
    pub global_routes: Arc<Mutex<Router<Context>>>,
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
    //print!("{}", Context::routing_info());

    let context = Context {
        plugin_info: None,
        global_routes: Arc::new(Mutex::new(Context::default_router())),
    };

    if let Ok(dir) = std::env::var("OVERLAY") {
        overlay_main(context, dir);
    } else {
        normal_main(context);
    }
}

fn normal_main(context: Context) {
    let cmd = std::env::args().nth(1).expect("1 Argument needed");

    let q = QueryOperation::And(vec![
        QueryOperation::ByTag(prelude::Tag::new("command", cmd.as_str())),
        QueryOperation::ByTag(prelude::Tag::new("type", "ELF")),
    ]);
    let entities = crate::core::entry::query((), q);
    let ent = entities.get(0).expect("Command Not Found!");

    let args = std::env::args().skip(1).collect::<Vec<String>>();
    runner::start_plugin_by_entry(&context, ent, &args)
        .unwrap()
        .wait();
}

fn overlay_main(context: Context, mut dir: String) {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::*;

    let (tx, rx) = channel();
    // Lifetime stuff
    let mut _tx_deamon: Option<mpsc::Sender<_>> = None;
    let mut _watcher_deamon: Option<RecommendedWatcher> = None;

    if let Ok(new_dir) = std::env::var("OVERLAY_WATCH") {
        println!("Using Overlay Watch...");
        println!("Load initial Overlay...");
        match util::directory_overlay::apply(&mut *(GLOBAL_STATE.write().unwrap()), &dir) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{:?}", e);
                std::process::exit(1);
            }
        };
        println!("Looping...");
        let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).unwrap();
        watcher.watch(&new_dir, RecursiveMode::Recursive).unwrap();
        dir = new_dir;
        _watcher_deamon = Some(watcher);
    } else {
        _tx_deamon = Some(tx);
    }

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

        let entities = crate::core::entry::query((), q);
        let ent = entities.get(0).expect("Command Not Found!");

        let args = std::env::args().skip(1).collect::<Vec<String>>();

        let pid = runner::start_plugin_by_entry(&context, ent, &args).unwrap();

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
