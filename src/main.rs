use log::LevelFilter;
use rtfw_http::{file_server::FileServer, router::Router, web_server::WebServer};

use config::Config;

mod config;
mod routes;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let config = Config::load_from_file()?;

    let file_server = FileServer::new()
        .map_file("/favicon.ico", "src/assets/favicon.ico")?
        .map_file("/main.css", "src/styles/main.css")?
        .map_dir("/static", "src/assets/")?
        .map_dir("/scripts", "src/scripts/")?;

    let router = Router::new()
        .get("/", routes::get_index)?
        .get("/home", routes::get_index)?
        .get("/send", routes::get_file)?
        .post("/send", routes::post_file)?
        .get("/paste", routes::get_paste)?
        .delete("/files", routes::delete_file)?
        .post("/paste", routes::post_paste)?
        .get("/mirror", routes::get_mirror)?
        .post("/mirror", routes::post_mirror)?
        .get("/slow", routes::get_slow)?
        // 404 Catch all
        .get("/*", routes::get_404)?
        .set_file_server(file_server);

    let server = WebServer::new(&config.hostname, router)?;
    server.run()
}
