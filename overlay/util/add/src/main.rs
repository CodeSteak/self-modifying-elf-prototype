extern crate ipc;
extern crate plugin;

pub use ipc::cbor::Value;
pub use ipc::*;
pub use plugin::*;

extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "add", about = "add data")]
enum Add {
    #[structopt(name = "kv")]
    KV { name: String, data: String },
    #[structopt(name = "tag")]
    Tag {
        name: String,
        tag: String,
        #[structopt(short = "v", long = "value")]
        value: Option<String>,
    },
    #[structopt(name = "file")]
    File {
        #[structopt(short = "n", long = "name")]
        name: Option<String>,
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

fn main() {
    let args: Add = StructOpt::from_args();

    let ctx = Channel::new_from_env(());

    match args {
        Add::KV { name, data } => {
            add_data(&ctx, &name, data.into());
            add_tag(
                &ctx,
                &name,
                plugin::tag_names::types::TAG,
                Some(plugin::tag_names::types::TEXT),
            )
        }
        Add::File { name, path } => {
            use std::fs::*;
            use std::io::prelude::*;

            let mut data = vec![];
            File::open(&path)
                .expect("Unable to open file!")
                .read_to_end(&mut data)
                .expect("Unable to read full file.");
            let name = name
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| path.as_os_str().to_str().unwrap());
            add_data(&ctx, name, data);
            if let Some(ext) = path
                .extension()
                .and_then(|o| o.to_owned().into_string().ok())
            {
                add_tag(&ctx, name, plugin::tag_names::types::TAG, Some(&ext));
            }
        }
        Add::Tag { name, tag, value } => {
            add_tag(&ctx, &name, &tag, value.as_ref().map(|d| d.as_str()));
        }
    }
}

fn add_data(ctx: &Channel, name: &str, data: Vec<u8>) {
    let hash_ref = HashRef::from_data(&data);

    assert_eq!(
        core::entry::write(&ctx, &WriteOperation::SmallData { data }),
        true
    );
    assert_eq!(
        core::entry::write(
            &ctx,
            &WriteOperation::Entry {
                old: None,
                new: Some(Entry {
                    name: name.to_owned(),
                    data: hash_ref,
                    tags: Default::default(),
                }),
            }
        ),
        true
    );
}

fn add_tag(ctx: &Channel, name: &str, tag: &str, tag_value: Option<&str>) {
    let mut new: Entry = core::entry::read(ctx, name).expect("Didn't find entry!");

    new.tags.insert(Tag::new(tag, tag_value));
    assert_eq!(
        core::entry::write(
            &ctx,
            &WriteOperation::Entry {
                old: None,
                new: Some(new),
            }
        ),
        true
    );
}
