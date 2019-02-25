extern crate ipc;
extern crate plugin;

pub use ipc::cbor::Value;
pub use ipc::*;
pub use plugin::*;

extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "export", about = "export and clean data")]
struct Export {
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    file: Option<PathBuf>,

    #[structopt(short = "e", long = "exclude-binary")]
    exclude_binary: bool,

    #[structopt(long = "force")]
    force: bool,
}

fn main() -> std::io::Result<()> {
    let args: Export = StructOpt::from_args();
    let ctx: Channel = Channel::new_from_env(());

    use std::fs::*;
    use std::io::*;

    let mut fop = OpenOptions::new();
    fop.write(true);
    if args.force {
        fop.create(true);
        fop.truncate(true);
    } else {
        fop.create_new(true);
    }

    let mut file = fop.open(args.file.unwrap_or(PathBuf::from("-")))?;

    if !args.exclude_binary {
        let bin: Option<ByteBuf> = core::own_executable::read(&ctx).ok();

        let bytes = bin.ok_or(Error::new(
            ErrorKind::NotFound,
            "Unable to read own_executable",
        ))?;

        file.write_all(bytes.as_ref())?;
    }

    let res = core::entry::list(&ctx);

    for e in res {
        println!("\t\t\t + {}", &e.name);

        let data = core::hash::read(&ctx, &e.data).ok_or(Error::new(
            ErrorKind::NotFound,
            "Unable to read data from entity.",
        ))?;;

        let wr_op = WriteOperation::SmallData { data: data.into() };
        cbor::to_writer(&mut file, &wr_op)
            .map_err(|_| Error::new(ErrorKind::Other, "Couldn't write data."))?;

        let ep_op = WriteOperation::Entry {
            old: None,
            new: Some(e),
        };

        cbor::to_writer(&mut file, &ep_op)
            .map_err(|_| Error::new(ErrorKind::Other, "Couldn't write entry."))?;
    }

    Ok(())
}
