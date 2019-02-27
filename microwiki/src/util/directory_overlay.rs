use crate::prelude::*;
use std::collections::*;
use std::fs::{self, *};
use std::io::Result;
use std::io::*;
use std::path::*;
use std::sync::*;

pub fn apply<P: Into<PathBuf>>(state: &mut State, path: P) -> Result<()> {
    walk_dir(path, |entry| {
        if entry.extension().and_then(|s| s.to_str()) == Some("entry") {
            println!("Loading {:?}", entry);
            parse_entry_file(state, entry)?;
        }

        Ok(())
    })?;

    println!("Done");

    Ok(())
}

pub fn walk_dir<F: FnMut(PathBuf) -> Result<()> + Sized, P: Into<PathBuf>>(
    path: P,
    mut f: F,
) -> Result<()> {
    let mut todo = vec![path.into()];

    while let Some(dir) = todo.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                todo.push(entry.path())
            } else if entry.file_type()?.is_file() {
                f(entry.path())?;
            }
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
const DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const DEBUG: bool = false;

// Don't try to read this code....
// This is for DEV purposes only.
pub fn parse_entry_file(state: &mut State, path: PathBuf) -> Result<()> {
    const INCLUDE_TOKEN: &str = "#include";
    const CARGO_TOKEN: &str = "#cargo";
    const DIRECTORY_TOKEN: &str = "#dir";
    const CONTENT_TOKEN: &str = "<<";

    let mut f = File::open(&path)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let mut lines = content.lines();

    while let Some(name) = lines.next() {
        if name.trim().is_empty() {
            continue;
        }
        let mut tags: BTreeSet<Tag> = Default::default();
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

                state
                    .data
                    .insert(hash.clone(), DataSource::Memory(Arc::new(data.into())));

                state.entries.insert(
                    name.clone(),
                    Entry {
                        name: name.clone(),
                        data: hash.clone(),
                        tags: tags.clone(),
                    },
                );

                break;
            } else if line.starts_with(DIRECTORY_TOKEN) {
                let arg = &line[DIRECTORY_TOKEN.len()..];
                let arg = arg.trim();

                let mut path = path.clone();
                path.pop(); // pop "*.entry" file
                path.push(arg);
                println!("Walking {:?}", &path);
                walk_dir(path, |path| {
                    let mut data = vec![];
                    println!("      * {:?}", &path);

                    let mut file = File::open(&path)?;
                    file.read_to_end(&mut data)?;

                    let hash = HashRef::from_data(&data);

                    state
                        .data
                        .insert(hash.clone(), DataSource::Memory(Arc::new(data.into())));

                    let name = format!("{}/{}", name, path.file_name().unwrap().to_str().unwrap());

                    state.entries.insert(
                        name.clone(),
                        Entry {
                            name: name.clone(),
                            data: hash.clone(),
                            tags: tags.clone(),
                        },
                    );
                    Ok(())
                })
                .expect("Failed adding files.");
            } else if line.starts_with(CARGO_TOKEN) {
                let arg = &line[CARGO_TOKEN.len()..];
                let arg = arg.trim();

                let mut path = path.clone();
                path.pop(); // pop this file
                path.push(arg);

                use std::process::*;

                let mut cargo = if DEBUG {
                    Command::new("cargo")
                        .arg("build")
                        .current_dir(path.clone())
                        .spawn()
                        .unwrap()
                } else {
                    Command::new("cargo")
                        .arg("build")
                        .arg("--release")
                        .current_dir(path.clone())
                        .spawn()
                        .unwrap()
                };

                if !cargo.wait()?.success() {
                    panic!("Aborting... Cargo returned with != 0");
                }

                // Fake Parse Cargo.toml #hack to get
                // name of binary
                let proj_name = {
                    let mut cargo_toml = path.clone();
                    cargo_toml.push("Cargo.toml");
                    let mut buf = String::new();
                    File::open(cargo_toml)
                        .expect("Didn't find Cargo.toml")
                        .read_to_string(&mut buf)
                        .expect("Cargo.toml has invalid UTF-8");

                    buf.lines().flat_map(|line| {
                        let mut split = line.splitn(2,"=").map(|s| s.trim());
                        if let (Some("name"), Some(value)) = (split.next(),split.next()) {
                            Some(value.replace("\"", ""))
                        }else {
                            None
                        }
                    }).next()
                        .expect(
                        "Didn't find name in Cargo.toml. Maybe this parser isn't advanced enough. \
                        Use this format: `name = \"my_plugin\"`.")
                };

                if DEBUG {
                    path.push("debug");
                } else {
                    path.push("release");
                }

                path.push(&proj_name);

                if !path.exists() {
                    // Fallback to path, in own workspace.

                    path = PathBuf::new();
                    path.push("target");
                    if DEBUG {
                        path.push("debug");
                    } else {
                        path.push("release");
                    }
                    path.push(&proj_name);
                }

                println!("{:?}", &path);

                if !DEBUG {
                    if let Ok(_) = std::env::var("USE_STRIP") {
                        let _ = Command::new("strip").arg(&path).spawn().unwrap().wait();
                    }

                    if let Ok(_) = std::env::var("USE_UPX") {
                        let _ = Command::new("upx")
                            .arg("--brute")
                            .arg(&path)
                            .spawn()
                            .unwrap()
                            .wait();
                    }
                }

                let mut data = vec![];
                let mut file = File::open(&mut path)?;
                file.read_to_end(&mut data)?;

                let hash = HashRef::from_data(&data);

                state
                    .data
                    .insert(hash.clone(), DataSource::Memory(Arc::new(data.into())));
                //println!("Adding via Cargo {}", &name);
                //println!("\t\tTags : {:?}", &tags);
                state.entries.insert(
                    name.clone(),
                    Entry {
                        name: name.clone(),
                        data: hash.clone(),
                        tags: tags.clone(),
                    },
                );

                break;
            } else if line.starts_with(CONTENT_TOKEN) {
                let arg = &line[CONTENT_TOKEN.len()..];
                let arg = arg.trim();

                let mut data = String::new();

                // Read until arg appears.
                while let Some(line) = lines.next() {
                    if line.trim() == arg {
                        break;
                    }
                    data.push_str(line);
                    data.push('\n');
                }

                let data: Vec<u8> = data.into();
                let hash = HashRef::from_data(&data);

                state
                    .data
                    .insert(hash.clone(), DataSource::Memory(Arc::new(data.into())));
                //println!("Adding {}", &name);
                //println!("\t\tTags : {:?}", &tags);
                state.entries.insert(
                    name.clone(),
                    Entry {
                        name: name.clone(),
                        data: hash.clone(),
                        tags: tags.clone(),
                    },
                );

                break;
            } else {
                let mut tag = line.trim().splitn(2, " ");
                match (tag.next(), tag.next()) {
                    (Some(a), value) if !a.is_empty() => {
                        tags.insert(Tag::new(a.trim(), value.map(|v| v.trim())));
                    }
                    (Some(_empty), _value) => {
                        // skip
                    }
                    (None, None) => {
                        // skip
                    }
                    (None, Some(_)) => {
                        unreachable!();
                    }
                }
            }
        }
    }

    Ok(())
}
