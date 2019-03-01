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
        #[structopt(short = "t", long = "tag")]
        tags: Vec<String>,
    },
    #[structopt(name = "elf")]
    Elf {
        #[structopt(short = "n", long = "name")]
        name: Option<String>,
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(short = "t", long = "tag")]
        tags: Vec<String>,
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
        Add::File { name, path, tags } => {
            let name = add_file(&ctx, &name, &path);
            for tag in tags {
                let mut tag_parts = tag.splitn(2, "=");
                add_tag(
                    &ctx,
                    &name,
                    tag_parts.next().unwrap().trim(),
                    tag_parts.next().map(|s| s.trim()),
                );
            }
        }
        Add::Elf { name, path, tags } => {
            let name = add_file(&ctx, &name, &path);
            let libs = find_dynlibs(&ctx, &name, &path);
            for lib in libs.into_iter() {
                println!("\tadded dyn lib {}", &lib);
                add_tag(&ctx, &name, "lib", Some(&lib));
            }
            for tag in tags {
                let mut tag_parts = tag.splitn(2, "=");
                add_tag(
                    &ctx,
                    &name,
                    tag_parts.next().unwrap().trim(),
                    tag_parts.next().map(|s| s.trim()),
                );
            }
            add_tag(&ctx, &name, "command", Some(&name));
            add_tag(&ctx, &name, "type", Some("elf"));
        }
        Add::Tag { name, tag, value } => {
            add_tag(&ctx, &name, &tag, value.as_ref().map(|d| d.as_str()));
        }
    }

    println!("Okay");
}

fn find_dynlibs(ctx: &Channel, prefix: &str, path: &PathBuf) -> Vec<String> {
    use std::process::*;

    let Output {
        stdout,
        status,
        stderr,
        ..
    } = Command::new("ldd")
        .arg(path)
        .output()
        .expect("Failed getting shared libs");

    if !status.success() {
        panic!("ldd failed : {}", String::from_utf8(stderr).unwrap());
    }

    String::from_utf8(stdout)
        .expect("Got invalid utf-8")
        .lines()
        .map(|l| l.trim())
        .flat_map(|l| {
            let mut parts = l.splitn(4, " ");
            let name = parts.next()?;
            let _dash = parts.next()?; // "=>"
            let path = parts.next()?;
            let _addr = parts.next()?;

            Some((
                format!("{}/{}", prefix, name).replace("//", "/"),
                path.clone(),
            ))
        })
        .map(|(name, path)| {
            add_file(ctx, &Some(name.clone()), &PathBuf::from(path));
            name
        })
        .collect::<Vec<_>>()
}

fn add_file(ctx: &Channel, name: &Option<String>, path: &PathBuf) -> String {
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

    name.to_string()
}

fn add_data(ctx: &Channel, name: &str, data: Vec<u8>) {
    let hash_ref = HashRef::from_data(&data);

    assert_eq!(core::hash::write(&ctx, &WriteSmallData { data }), true);
    assert_eq!(
        core::entry::write(
            &ctx,
            &WriteEntry {
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
            &WriteEntry {
                old: Some(name.to_string()),
                new: Some(new),
            }
        ),
        true
    );
}
