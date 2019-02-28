use crate::prelude::*;
use std::path::PathBuf;

fn temp_path() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("microwiki");
    temp
}

pub(crate) fn delete_temp_dirs() {
    let _ = std::fs::remove_dir_all(temp_path());
}

pub(crate) fn setup_ld_path(_context: &Context, entry: &Entry) -> Option<String> {
    let mut temp = temp_path();
    temp.push(&entry.name.replace("/", "-").replace(".", "--"));

    let mut ret = None;
    for lib in entry
        .tags
        .iter()
        .filter(|tag| tag.name == "lib")
        .flat_map(|tag| tag.value.clone())
    {
        use std::fs::*;
        use std::io::*;

        DirBuilder::new()
            .recursive(true)
            .create(&temp)
            .expect("Unable to create director in temp");

        let lib_entry = crate::core::entry::read((), lib.clone()).expect("Didn't find lib.");

        let data = crate::core::hash::read((), lib_entry.data.clone()).expect("No data for lib.");

        let file_name = lib_entry.name.split("/").last().unwrap().to_string();
        let mut file_path = temp.clone();
        file_path.push(&file_name);
        {
            use std::os::unix::fs::PermissionsExt;

            let mut file = File::create(&file_path).expect("Unable to create file in temp.");

            file.write_all(&data[..])
                .expect("Unable to write to file in temp.");

            let metadata = file.metadata().expect("Cannot get metadata");
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o700);
        }

        ret = Some(temp.clone().into_os_string().into_string().unwrap());
    }

    ret
}
