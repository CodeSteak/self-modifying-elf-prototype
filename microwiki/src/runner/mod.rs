use ipc::bs::*;
use ipc::*;
use plugin::*;

use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;

use crate::Context;

use std::sync::*;

mod elf_util;

#[derive(Clone)]
pub struct PluginInfo {
    pub call_channel: Arc<Mutex<BufStream<UnixStream>>>,
}

pub struct ChildPid {
    pid: nix::unistd::Pid,
}

impl ChildPid {
    pub fn wait(self) {
        use nix::sys::wait::*;
        let _ = waitpid(self.pid, None);
    }
    pub fn kill(self) {
        use nix::sys::signal::*;

        let _ = kill(self.pid, SIGINT);
        self.wait()
    }
}

impl Drop for ChildPid {
    fn drop(&mut self) {
        elf_util::delete_temp_dirs();
    }
}

pub fn start_plugin_by_entry(
    context: &Context,
    entry: &Entry,
    args: &[String],
) -> Option<ChildPid> {
    let data = crate::core::hash::read((), entry.data.clone())?;
    use nix::sys::memfd::*;
    use nix::unistd::*;
    use std::ffi::*;

    let mut context = context.clone();

    let mem_fd_name = CString::new("mcwk-srv").unwrap();
    let mem_fd = memfd_create(
        mem_fd_name.as_c_str(), // TODO?
        MemFdCreateFlag::empty(),
    )
    .expect("memfd_create failed!");

    let written = write(mem_fd, &data.as_ref()[..]).expect("Write to mem_fd failed!");
    if written != data.as_ref().len() {
        panic!(
            "Writting to memfd failed! {} != {}",
            written,
            data.as_ref().len()
        );
    }

    // p2h: Plugin to Host.
    // h2p: Host to Plugin.
    let (p2h_host, p2h_plugin) = UnixStream::pair().unwrap();
    let (h2p_host, h2p_plugin) = UnixStream::pair().unwrap();

    let h2p_host = Arc::new(Mutex::new(BufStream::new(h2p_host)));

    let info = PluginInfo {
        call_channel: h2p_host.clone(),
    };

    context.plugin_info = Some(info);

    let ld_path_extra = elf_util::setup_ld_path(&context, entry)
        .map(|ld_path| ("LD_LIBRARY_PATH".to_string(), ld_path.clone()));

    Channel::new_as_host((p2h_host, h2p_host), context.global_routes.clone(), context);

    set_no_close_exec(&p2h_plugin);
    set_no_close_exec(&h2p_plugin);

    match fork() {
        Ok(ForkResult::Parent { child }) => {
            let _ = close(mem_fd);
            Some(ChildPid { pid: child })
        }
        Ok(ForkResult::Child) => {
            let args: Vec<CString> = args
                .iter()
                .map(|s| CString::new(s.clone()).unwrap())
                .collect();

            let extra_env = vec![
                (
                    "PluginToHost_FD".to_string(),
                    format!("{}", p2h_plugin.as_raw_fd()),
                ),
                (
                    "HostToPlugin_FD".to_string(),
                    format!("{}", h2p_plugin.as_raw_fd()),
                ),
            ];

            let env: Vec<CString> = extra_env
                .into_iter()
                .chain(ld_path_extra.into_iter())
                .chain(std::env::vars())
                .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
                .collect();

            let _e = fexecve(mem_fd, &args, &env);
            panic!("Exec failed! ");
        }
        Err(e) => {
            panic!("Fork failed! {} ", e);
        }
    }
}

fn set_no_close_exec<S: AsRawFd>(fd: &S) {
    use nix::fcntl::*;

    let ret = fcntl(fd.as_raw_fd(), FcntlArg::F_GETFD).expect("Failed getting CLOEXEC");

    let mut flags = FdFlag::from_bits(ret).unwrap();
    flags.remove(FdFlag::FD_CLOEXEC);

    fcntl(fd.as_raw_fd(), FcntlArg::F_SETFD(flags)).expect("Failed setting CLOEXEC");
}
