use anyhow::{anyhow, Result};
use log::debug;
use regex::Regex;
use std::{
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
};

const UPLOAD_DIR: &str = "data/upload/";
const PASTE_FILE: &str = "data/shared-text.txt";

pub fn is_safe_relative_subpath(path: &Path) -> bool {
    !path.is_absolute() && path.components().all(|comp| comp != Component::ParentDir)
}

pub fn get_upload_dir_path() -> PathBuf {
    PathBuf::from(UPLOAD_DIR)
}

pub fn markdown_to_html(content: &str) -> Result<String> {
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;
    let result = link_regex
        .replace_all(content, r#"<a href="$2">$1</a>"#)
        .to_string();

    let bold_regex = Regex::new(r"\*\*(.*?)\*\*")?;
    let result = bold_regex.replace_all(&result, "<b>$1</b>").to_string();

    let italic_regex = Regex::new(r"\*(.*?)\*")?;
    let result = italic_regex.replace_all(&result, "<i>$1</i>").to_string();

    let code_regex = Regex::new(r"`(.*?)`")?;
    let result = code_regex
        .replace_all(&result, "<code>$1</code>")
        .to_string();

    Ok(result)
}

pub fn get_upload_file_path(filename: &str) -> Result<PathBuf> {
    let safe_filename = sanitize_filename(filename)?;
    Ok(get_upload_dir_path().join(safe_filename))
}

pub fn sanitize_filename(filename: &str) -> Result<OsString> {
    match Path::new(filename).file_name() {
        Some(filename) => Ok(filename.to_owned()),
        None => Err(anyhow!("failed to get filename")),
    }
}

pub fn get_paste_data() -> Result<String> {
    debug!("get paste data from: {}", PASTE_FILE);
    Ok(fs::read_to_string(PASTE_FILE)?)
}

pub fn write_paste_data(data: &str) -> Result<()> {
    debug!("write to {}: {}", PASTE_FILE, data);
    fs::write(PASTE_FILE, data)?;
    Ok(())
}

pub fn load_view(name: &str) -> Result<String> {
    let view_path = format!("src/views/{}.html", name);
    let content = fs::read_to_string(view_path)?;
    Ok(content)
}

pub fn sanitize_user_input(value: &str) -> String {
    value.replace("<", "&lt;").replace(">", "&gt;")
}

pub fn get_files_in_directory(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(path)?;
    let files = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    Ok(files)
}
