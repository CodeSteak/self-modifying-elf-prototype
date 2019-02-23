use crate::prelude::*;
use std::fs::{self, *};
use std::path::*;
use std::io::*;
use std::io::Result;
use std::collections::*;
use std::sync::*;

pub fn apply<P : Into<PathBuf>>(state : &mut State, path : P) -> Result<()> {
    let mut todo = vec![path.into()];

    while let Some(dir) = todo.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                todo.push(entry.path())
            } else if entry.path().extension().and_then(|s| s.to_str()) == Some("entry") {
                //println!("Loading {:?}", entry.path());
                parse_entry_file(state, entry.path())?;
            }
        }
    }

    Ok(())
}

// Don't try to read this code....
// This is for DEV purposes only.
pub fn parse_entry_file(state : &mut State, path : PathBuf) -> Result<()> {
    const INCLUDE_TOKEN : &str = "#include";
    const CARGO_TOKEN : &str = "#cargo";
    const CONTENT_TOKEN : &str = "<<";

    let mut f = File::open(&path)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let mut lines = content.lines();

    while let Some(name) = lines.next() {
        if name.trim().is_empty() { continue; }
        let mut tags : BTreeSet<Tag> = Default::default();
        let name = name.trim().to_owned();

        while let Some(line) = lines.next() {
            if line.starts_with(INCLUDE_TOKEN) {
                let arg = &line[INCLUDE_TOKEN.len()..];
                let arg = arg.trim();

                let mut path = path.clone();
                path.pop(); // pop this file
                path.push(arg);
                //println!("Reading {:?}", path);

                let mut data = vec![];
                let mut file = File::open(path)?;
                file.read_to_end(&mut data)?;

                let hash = HashRef::from_data(&data);

                state.data.insert(hash.clone(), DataSource::Memory( Arc::new(data.into() )));
                //println!("Adding via File {}", &name);
                //println!("\t\tTags : {:?}", &tags);
                state.entries.insert(name.clone(), Entry {
                    name: name.clone(),
                    data: hash.clone(),
                    tags: tags.clone(),
                });

                break;
            }else if line.starts_with(CARGO_TOKEN) {
                let arg = &line[CARGO_TOKEN.len()..];
                let arg = arg.trim();

                let mut path = path.clone();
                path.pop(); // pop this file
                path.push(arg);
                //println!("Cargoing {:?}", &path);

                use std::process::*;

                let mut cargo = Command::new("cargo")
                    .arg("build")
                    .arg("--release")
                    .current_dir(path.clone())
                    .spawn()
                    .unwrap();
                if !cargo.wait()?.success() {
                    panic!("Aborting... Cargo returned with != 0");
                }

                let proj_name = path.components().last().unwrap().as_os_str().to_owned();
                path.push("target");
                path.push("release");
                path.push(proj_name);

                let mut data = vec![];
                let mut file = File::open(&mut path)?;
                file.read_to_end(&mut data)?;

                let hash = HashRef::from_data(&data);

                state.data.insert(hash.clone(), DataSource::Memory( Arc::new(data.into() )));
                //println!("Adding via Cargo {}", &name);
                //println!("\t\tTags : {:?}", &tags);
                state.entries.insert(name.clone(), Entry {
                    name: name.clone(),
                    data: hash.clone(),
                    tags: tags.clone(),
                });

                break;
            }else if line.starts_with(CONTENT_TOKEN) {
                let arg = &line[CONTENT_TOKEN.len()..];
                let arg = arg.trim();

                let mut data = String::new();

                // Read until arg appears.
                while let Some(line) = lines.next() {
                    if line.trim() == arg { break; }
                    data.push_str(line);
                    data.push('\n');
                }

                let data : Vec<u8> = data.into();
                let hash = HashRef::from_data(&data);

                state.data.insert(hash.clone(), DataSource::Memory( Arc::new(data.into() )));
                //println!("Adding {}", &name);
                //println!("\t\tTags : {:?}", &tags);
                state.entries.insert(name.clone(), Entry {
                    name: name.clone(),
                    data: hash.clone(),
                    tags: tags.clone(),
                });

                break;
            }else {
                let mut tag = line.trim().splitn(2, " ");
                match (tag.next(), tag.next()) {

                    (Some(a), value) if !a.is_empty() => {
                        tags.insert(Tag::new(a, value));
                    },
                    (Some(a), value) => {
                        // skip
                    }
                    (None,None) => {
                        // skip
                    },
                    (None, Some(_)) => {
                        unreachable!();
                    }
                }
            }
        }
    }

    Ok(())
}