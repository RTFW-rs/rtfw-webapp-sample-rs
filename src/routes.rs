use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use log::{debug, warn};
use rand::seq::IndexedRandom;
use rtfw_http::http::response_status_codes::HttpStatusCode;
use rtfw_http::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

use crate::config::Config;
use crate::utils;

pub fn get_index(request: &HttpRequest) -> Result<HttpResponse> {
    let name = request.query.get("name").map_or("World", |v| v);
    let greet_msg = format!("Hello {}!", name);
    debug!("greeting: {greet_msg}");

    let body = utils::load_view("index")?.replace("{{GREET_MSG}}", &greet_msg);
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn post_paste(request: &HttpRequest) -> Result<HttpResponse> {
    let config = Config::load_from_file()?;

    let mut body = request.get_str_body()?;
    if let Some(max_paste_size) = config.paste.max_paste_size {
        if body.len() > max_paste_size {
            return HttpResponseBuilder::new()
                .set_status(HttpStatusCode::BadRequest)
                .set_json_body(
                    &json!({"status": "400 Bad Request", "message": "too many characters!"}),
                )?
                .build();
        }
    }

    if !config.paste.allow_html_injection {
        body = utils::sanitize_user_input(&body);
    }

    utils::write_paste_data(&body)?;

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::OK)
        .set_json_body(&json!({"status": "200 OK", "message": "data has been saved"}))?
        .build()
}

pub fn delete_file(request: &HttpRequest) -> Result<HttpResponse> {
    let filename = request.get_str_body()?;
    let file_path = utils::get_upload_file_path(&filename)?;
    if !file_path.exists() {
        let response = json!({"status": "404 Not Found", "message": "no such file"});
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .set_json_body(&response)?
            .build();
    }

    fs::remove_file(file_path)?;
    let response = json!({"status": "200 OK", "message": "file was deleted"});
    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::OK)
        .set_json_body(&response)?
        .build()
}

pub fn get_paste(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = utils::load_view("paste")?;
    let data = utils::get_paste_data()?;
    let body = body.replace("{{PASTE}}", &data);

    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    debug!("someone is mirroring their request: {:?}", request);
    HttpResponseBuilder::new().set_json_body(request)?.build()
}

pub fn post_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    let body = request.get_str_body()?;
    debug!("someone is mirroring their request: {:?}", request);
    debug!("limit=============");
    debug!("body: {body}");
    debug!("limit=============");
    HttpResponseBuilder::new().set_json_body(request)?.build()
}

fn get_files_html(files: &[PathBuf]) -> Result<String> {
    let mut dynamic_html = String::new();
    for file in files.iter() {
        let metadata = fs::metadata(file)?;
        let created_time = DateTime::<Local>::from(metadata.created()?).format("%Y-%m-%d %H:%M:%S");

        let filename = file
            .file_name()
            .context("file should have a name")?
            .to_string_lossy();

        let file_url = format!("/send?file={}", filename);

        let filename_display = format!("<span class=\"file-name\"> {filename}</span>");
        let date = format!("<span class=\"file-upload-date\"> {created_time}</span>");
        let view_btn = format!(
            "<button class=\"view-file\" data-url=\"{file_url}\" title=\"View\"></button>"
        );
        let delete_btn =
            format!("<button class=\"delete-file\" data-filename=\"{filename}\" title=\"Delete\"></button>");

        let file_dom_el = format!("<li>{filename_display}{date}{view_btn}{delete_btn}</li>");
        dynamic_html.push_str(&file_dom_el);
    }

    Ok(dynamic_html)
}

pub fn get_file(request: &HttpRequest) -> Result<HttpResponse> {
    let filename = request.query.get("file");
    if filename.is_none() {
        debug!("no filename provided, showing index view");
        let body = utils::load_view("send")?;
        let upload_dir = utils::get_upload_dir_path();
        let files = utils::get_files_in_directory(&upload_dir)?;

        let dynamic_html = get_files_html(&files)?;
        let body = body.replace("{{FILES}}", &dynamic_html);
        return HttpResponseBuilder::new().set_html_body(&body).build();
    }

    let filename = filename.unwrap();
    debug!("filename request: {filename}");

    let filepath = utils::get_upload_file_path(filename)?;
    if !filepath.is_file() {
        let err_msg = format!("No such file: {}", filename);
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .set_json_body(&json!({"status": "404 Not Found", "message": err_msg}))?
            .build();
    }

    debug!("file returned: {:?}", filepath);
    let mime_type = mime_guess::from_path(&filepath).first_or_octet_stream();
    let bin_content = fs::read(&filepath)?;

    HttpResponseBuilder::new()
        .set_raw_body(bin_content)
        .set_content_type(mime_type.as_ref())
        .build()
}

pub fn post_file(request: &HttpRequest) -> Result<HttpResponse> {
    let config = Config::load_from_file()?;

    let multipart = request.get_multipart_body()?;
    let file_part = multipart
        .parts
        .first()
        .context("MultiPart body should not be empty")?;

    if let Some(max_file_size) = config.send.max_file_size {
        if file_part.data.len() > max_file_size {
            let message = format!("file is too big, MAX: {} bytes", max_file_size);
            return HttpResponseBuilder::new()
                .set_status(HttpStatusCode::BadRequest)
                .set_json_body(&json!({"status": "400 Bad Request", "message": message}))?
                .build();
        }
    }

    if let Some(file_limit) = config.send.file_limit {
        let upload_dir = utils::get_upload_dir_path();
        let uploaded_files = utils::get_files_in_directory(&upload_dir)?;
        if uploaded_files.len() >= file_limit {
            let message = format!(
                "Cannot upload file because limit has been reached: {} files",
                file_limit
            );

            return HttpResponseBuilder::new()
                .set_status(HttpStatusCode::BadRequest)
                .set_json_body(&json!({"status": "400 Bad Request", "message": message}))?
                .build();
        }
    }

    let filename = file_part.filename.clone().unwrap();
    debug!("uploading file: {filename}");

    let filepath = utils::get_upload_file_path(&filename);
    if let Err(e) = filepath {
        warn!("failed to upload: {:?}: {}", multipart, e);
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::BadRequest)
            .set_json_body(
                &json!({"status": "400 Bad Request", "message": "failed to upload file"}),
            )?
            .build();
    }

    let filepath = filepath.unwrap();
    if filepath.exists() {
        warn!(
            "failed to upload: {:?}: file with same name already exists",
            multipart
        );
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::Conflict)
            .set_json_body(&json!({"status": "409 Conflict", "message": "A file with the same name already exists"}))?
            .build();
    }

    let mut file = File::create(&filepath)?;
    file.write_all(&file_part.data)?;
    debug!("saved data to file: {}", filepath.display());

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Created)
        .set_json_body(&json!({"status": "201 Created", "message": "file was uploaded to server"}))?
        .build()
}

pub fn get_404(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = utils::load_view("404")?;
    let catchphrases: Vec<_> = fs::read_to_string("src/assets/404_phrases.md")?
        .lines()
        .map(String::from)
        .collect();

    let phrase = catchphrases.choose(&mut rand::rng()).unwrap();
    let rendered_phrase = utils::markdown_to_html(phrase)?;
    let body = body.replace("{{CATCHPHRASE}}", &rendered_phrase);
    debug!(
        "someone got lost, giving them the catch all route and a catchphrase: {rendered_phrase}"
    );

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::NotFound)
        .set_html_body(&body)
        .build()
}

pub fn get_slow(request: &HttpRequest) -> Result<HttpResponse> {
    let time = request.query.get("time").map_or("2", |v| v).parse()?;
    let body = format!("Slept for {} seconds", time);

    debug!("someone is slowing down the server by: {time}s");
    thread::sleep(Duration::from_secs(time));
    HttpResponseBuilder::new().set_html_body(&body).build()
}
