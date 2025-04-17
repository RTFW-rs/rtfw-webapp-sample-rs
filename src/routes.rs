use anyhow::{Context, Result};
use log::warn;
use rand::seq::IndexedRandom;
use rtfw_http::http::response_status_codes::HttpStatusCode;
use rtfw_http::http::{HttpRequest, HttpResponse, HttpResponseBuilder};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, thread};

use crate::utils;

const PASTE_FILE: &str = "data/shared-text.txt";
const PASTE_UPLOAD_SIZE_LIMIT: usize = 1_000_000_000;

pub fn get_hello(request: &HttpRequest) -> Result<HttpResponse> {
    let name = request.query.get("name").map_or("World", |v| v);
    let greet_msg = format!("Hello {}!", name);

    let body = utils::load_view("index")?.replace("{{GREET_MSG}}", &greet_msg);
    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_favicon(_request: &HttpRequest) -> Result<HttpResponse> {
    let favicon = fs::read("src/assets/favicon.ico")?;
    HttpResponseBuilder::new()
        .set_raw_body(favicon)
        .set_content_type("image/x-icon")
        .build()
}

pub fn post_paste(request: &HttpRequest) -> Result<HttpResponse> {
    let body = request.str_body()?;
    // if body.len() > PASTE_UPLOAD_SIZE_LIMIT {
    //     return HttpResponseBuilder::new()
    //         .set_status(HttpStatusCode::BadRequest)
    //         .set_json_body(
    //             &json!({"status": "400 Bad Request", "message": "too many characters!"}),
    //         )?
    //         .build();
    // }

    // let body = utils::sanitize_user_input(&body);

    utils::write_paste_data(&body)?;

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::OK)
        .set_json_body(&json!({"status": "200 OK", "message": "data has been saved"}))?
        .build()
}

pub fn get_paste(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = utils::load_view("paste")?;
    let data = utils::get_paste_data()?;
    let body = body.replace("{{PASTE}}", &data);

    HttpResponseBuilder::new().set_html_body(&body).build()
}

pub fn get_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    HttpResponseBuilder::new().set_json_body(request)?.build()
}

pub fn post_mirror(request: &HttpRequest) -> Result<HttpResponse> {
    let body = request.str_body()?;
    println!("limit=============");
    println!("body: {body}");
    println!("limit=============");
    HttpResponseBuilder::new().set_json_body(request)?.build()
}

pub fn get_file(request: &HttpRequest) -> Result<HttpResponse> {
    let filename = request.query.get("file");
    if filename.is_none() {
        let body = utils::load_view("send")?;
        let upload_dir = utils::get_upload_dir_path();
        let files = utils::get_files_in_directory(&upload_dir)?;

        let mut dynamic_html = String::new();
        for file in files.iter() {
            // do not list 'secret' files
            if file.ends_with(".secret") {
                continue;
            };

            let url = format!("/send?file={}", file);
            let file_dom_el = format!("<li><a href=\"{url}\">{file}</a></li>");
            dynamic_html.push_str(&file_dom_el);
        }

        let body = body.replace("{{FILES}}", &dynamic_html);
        return HttpResponseBuilder::new().set_html_body(&body).build();
    }

    let filename = filename.unwrap();
    let filepath = utils::get_upload_file_path(filename)?;
    if !filepath.is_file() {
        let err_msg = format!("No such file: {}", filename);
        return HttpResponseBuilder::new()
            .set_status(HttpStatusCode::NotFound)
            .set_json_body(&json!({"status": "404 Not Found", "message": err_msg}))?
            .build();
    }

    let mime_type = mime_guess::from_path(&filepath).first_or_octet_stream();
    let bin_content = fs::read(&filepath)?;

    HttpResponseBuilder::new()
        .set_raw_body(bin_content)
        .set_content_type(&mime_type.to_string())
        .build()
}

pub fn post_file(request: &HttpRequest) -> Result<HttpResponse> {
    let content_type = request
        .headers
        .get("Content-Type")
        .context("expects Content-Type header")?;

    let boundary = content_type
        .value
        .strip_prefix("multipart/form-data; boundary=")
        .context("Should have prefix")?;

    let multipart = utils::process_multipart_form_data(boundary, &request.body)?;
    let filepath = utils::get_upload_file_path(&multipart.filename);
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

    let mut file = File::create(filepath)?;
    file.write_all(&multipart.data);

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::Created)
        .set_json_body(&json!({"status": "201 Created", "message": "file was uploaded to server"}))?
        .build()
}

pub fn get_404(_request: &HttpRequest) -> Result<HttpResponse> {
    let body = utils::load_view("404")?;
    let catchphrases: Vec<_> = fs::read_to_string("src/assets/404_phrases.txt")?
        .lines()
        .map(String::from)
        .collect();

    let phrase = catchphrases.choose(&mut rand::rng()).unwrap();
    let body = body.replace("{{CATCHPHRASE}}", phrase);

    HttpResponseBuilder::new()
        .set_status(HttpStatusCode::NotFound)
        .set_html_body(&body)
        .build()
}

pub fn get_slow(request: &HttpRequest) -> Result<HttpResponse> {
    let time = request.query.get("time").map_or("2", |v| v).parse()?;
    let body = format!("Slept for {} seconds", time);

    thread::sleep(Duration::from_secs(time));
    HttpResponseBuilder::new().set_html_body(&body).build()
}
