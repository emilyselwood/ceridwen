extern crate ureq;
extern crate walkdir;

use std::env;
use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // locate executable path even if the project is in workspace
    let executable_path = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").unwrap());

    // Download fonts and put into resources/static/fonts
    let fonts_dir = manifest_dir.join("resources").join("static").join("fonts");

    download(
        "https://github.com/NDISCOVER/Cinzel/raw/master/fonts/ttf/Cinzel-Regular.ttf",
        &fonts_dir.join("Cinzel-Regular.ttf"),
    )
    .unwrap();

    download(
        "https://github.com/CatharsisFonts/Cormorant/raw/master/fonts/ttf/Cormorant-Light.ttf",
        &fonts_dir.join("Cormorant-Light.ttf"),
    )
    .unwrap();

    copy(&manifest_dir.join("resources"), &executable_path);
}

fn locate_target_dir_from_output_dir(mut target_dir_search: &Path) -> Option<&Path> {
    loop {
        // if path ends with "target", we assume this is correct dir
        if target_dir_search.ends_with("target") {
            return Some(target_dir_search);
        }

        // otherwise, keep going up in tree until we find "target" dir
        target_dir_search = match target_dir_search.parent() {
            Some(path) => path,
            None => break,
        }
    }

    None
}

fn copy(from: &Path, to: &Path) {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.into();
    for entry in WalkDir::new(from_path.clone()) {
        let entry = entry.unwrap();

        if let Ok(rel_path) = entry.path().strip_prefix(&from_path) {
            let target_path = to_path.join(rel_path);

            if entry.file_type().is_dir() {
                DirBuilder::new()
                    .recursive(true)
                    .create(target_path)
                    .expect("failed to create target dir");
            } else {
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    }
}

fn download(from: &str, to: &Path) -> Result<(), String> {
    // exit if the file already exists.
    if to.exists() {
        return Ok(());
    }

    // make sure the target directory exists.
    if !to.parent().unwrap().exists() {
        fs::create_dir_all(to.parent().unwrap()).map_err(|error| error.to_string())?;
    }

    let response = ureq::get(from).call().map_err(|error| error.to_string())?;

    // Only accept 2xx status codes
    if !(200..300).contains(&response.status()) {
        return Err(format!("Download error: HTTP {}", response.status()));
    }

    let mut body = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut body)
        .map_err(|error| error.to_string())?;

    // write the file into the source tree.
    fs::write(to, body).map_err(|error| error.to_string())?;

    Ok(())
}
