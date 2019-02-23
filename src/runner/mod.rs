use ipc::*;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;

use crate::ROUTING_TABLE;

use ipc::bs::*;
use std::sync::*;
use plugin::data::hash_ref::HashRef;
use plugin::data::Entry;
use nix::sys::wait::waitpid;

#[derive(Clone, Default)]
pub struct PluginInfo {
    pub call_channel: Option<Arc<Mutex<BufStream<UnixStream>>>>,
}

pub fn run_async<F: FnOnce(Channel) + Send + Sized + 'static>(f: F) -> PluginInfo {
    run_blocking(|ch| {
        use std::thread;

        thread::spawn(move || f(ch));
    })
}

pub fn run_blocking<F: FnOnce(Channel) + Sized + 'static>(f: F) -> PluginInfo {
    // p2h: Plugin to Host.
    // h2p: Host to Plugin.
    let (p2h_host, p2h_plugin) = UnixStream::pair().unwrap();
    let (h2p_host, h2p_plugin) = UnixStream::pair().unwrap();

    let h2p_host = Arc::new(Mutex::new(BufStream::new(h2p_host)));

    let info = PluginInfo {
        call_channel: Some(h2p_host.clone()),
    };

    Channel::new_full((p2h_host, h2p_host), ROUTING_TABLE.clone(), info.clone());

    let plugin_channel = Channel::new((h2p_plugin, p2h_plugin));

    f(plugin_channel);

    info
}

pub fn start_plugin_via_cargo(sub_dir: &str, args: &[String]) {
    // p2h: Plugin to Host.
    // h2p: Host to Plugin.
    let (p2h_host, p2h_plugin) = UnixStream::pair().unwrap();
    let (h2p_host, h2p_plugin) = UnixStream::pair().unwrap();

    let h2p_host = Arc::new(Mutex::new(BufStream::new(h2p_host)));

    let info = PluginInfo {
        call_channel: Some(h2p_host.clone()),
    };

    Channel::new_full((p2h_host, h2p_host), ROUTING_TABLE.clone(), info.clone());

    use std::process::Command;

    set_no_close_exec(&p2h_plugin);
    set_no_close_exec(&h2p_plugin);

    let mut cmd = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .args(args.iter().skip(1))
        .current_dir(&sub_dir)
        .env("PluginToHost_FD", format!("{}", p2h_plugin.as_raw_fd()))
        .env("HostToPlugin_FD", format!("{}", h2p_plugin.as_raw_fd()))
        .spawn()
        .unwrap();

    cmd.wait().expect("failed to wait on child");
}

pub fn start_plugin_by_entry(entry : &Entry, args: &[String]) -> Option<()> {

    let data= crate::core::hash::read((entry.data.clone(),))?;
    use nix::sys::memfd::*;
    use std::ffi::*;
    use nix::unistd::*;

    let mem_fd_name = CString::new("mcwk-srv").unwrap();
    let mem_fd = memfd_create(
        mem_fd_name.as_c_str(), // TODO?
        MemFdCreateFlag::empty()).expect("memfd_create failed!");

    let written = write(mem_fd,&data.as_ref()[..]).expect("Write to mem_fd failed!");
    if written != data.as_ref().len() {
        panic!("Writting to memfd failed! {} != {}", written, data.as_ref().len());
    }

    // p2h: Plugin to Host.
    // h2p: Host to Plugin.
    let (p2h_host, p2h_plugin) = UnixStream::pair().unwrap();
    let (h2p_host, h2p_plugin) = UnixStream::pair().unwrap();

    let h2p_host = Arc::new(Mutex::new(BufStream::new(h2p_host)));

    let info = PluginInfo {
        call_channel: Some(h2p_host.clone()),
    };

    Channel::new_full((p2h_host, h2p_host), ROUTING_TABLE.clone(), info.clone());

    use std::process::Command;

    set_no_close_exec(&p2h_plugin);
    set_no_close_exec(&h2p_plugin);


    match fork() {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None);
        },
        Ok(ForkResult::Child) => {
            //Todo .env("PluginToHost_FD", format!("{}", p2h_plugin.as_raw_fd()))
            //        .env("HostToPlugin_FD", format!("{}", h2p_plugin.as_raw_fd()))
            let mut args : Vec<CString> = args.iter().map(|s| CString::new(s.clone()).unwrap()).collect();
            //args.push(CString::new("").unwrap());

            let extra_env = vec![
                ("PluginToHost_FD".to_string(),  format!("{}",p2h_plugin.as_raw_fd())),
                ("HostToPlugin_FD".to_string(),  format!("{}",h2p_plugin.as_raw_fd())),
            ];

            let mut env : Vec<CString> =  extra_env.into_iter().chain(std::env::vars()).map(|(k,v)|{
                CString::new(format!("{}={}",k,v)).unwrap()
            }).collect();

            fexecve(mem_fd, &args, &env);

            panic!("Exec failed!");
        },
        Err(e) => {
            panic!("Fork failed! {} ", e);
        }
    }

    close(mem_fd);

    Some(())
}

fn set_no_close_exec<S: AsRawFd>(fd: &S) {
    use nix::fcntl::*;

    let ret = fcntl(fd.as_raw_fd(), FcntlArg::F_GETFD).expect("Failed getting CLOEXEC");

    let mut flags = FdFlag::from_bits(ret).unwrap();
    flags.remove(FdFlag::FD_CLOEXEC);

    fcntl(fd.as_raw_fd(), FcntlArg::F_SETFD(flags)).expect("Failed setting CLOEXEC");
}
