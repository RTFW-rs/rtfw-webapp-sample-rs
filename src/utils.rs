use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Cursor, Read},
    path::{Path, PathBuf},
};

const UPLOAD_DIR: &str = "data/upload/";
const PASTE_FILE: &str = "data/shared-text.txt";

pub fn get_upload_dir_path() -> PathBuf {
    PathBuf::from(UPLOAD_DIR)
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
    info!("get paste data from: {}", PASTE_FILE);
    Ok(fs::read_to_string(PASTE_FILE)?)
}

pub fn write_paste_data(data: &str) -> Result<()> {
    info!("write to {}: {}", PASTE_FILE, data);
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

pub fn get_files_in_directory(path: &PathBuf) -> Result<Vec<String>> {
    let entries = fs::read_dir(path)?;
    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();
    Ok(file_names)
}

#[derive(Debug)]
pub struct MultipartFileDescriptor {
    pub name: String,
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

pub fn process_multipart_form_data(boundary: &str, data: &[u8]) -> Result<MultipartFileDescriptor> {
    let cursor = Cursor::new(data);
    let mut reader = BufReader::new(cursor);

    let mut read_boundary = String::new();
    reader.read_line(&mut read_boundary);

    info!("{}", read_boundary);
    info!("{}", boundary);
    if !read_boundary.eq(boundary) {
        warn!("Boundaries differ, not sure why, we'll see later");
        // return Err(anyhow!("boundary should be same"));
    }

    let mut content_disposition = String::new();
    reader.read_line(&mut content_disposition);
    info!("{}", content_disposition);

    let mut parts = content_disposition.split(";").map(|p| p.trim());
    let first = parts.next().context("should be form-data")?;
    let name = parts.next().context("should be name=")?.to_owned();
    let filename = parts
        .next()
        .context("should be filename=")?
        .split_once('=')
        .context("should have= in filename")?
        .1
        .replace("\"", "");

    let mut content_type = String::new();
    reader.read_line(&mut content_type);
    info!("{}", content_type);

    let mut empty_line = String::new();
    reader.read_line(&mut empty_line);

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer);
    buffer.truncate(buffer.len() - boundary.len() - 8);

    Ok(MultipartFileDescriptor {
        name,
        filename,
        content_type,
        data: buffer,
    })
}
