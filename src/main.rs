use log::LevelFilter;
use rtfw_http::{file_server::FileServer, router::Router, web_server::WebServer};

mod routes;
mod utils;

const HOSTNAME: &str = "127.0.0.1:7878";

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    let file_server = FileServer::new()
        .map_file("/favicon.ico", "src/assets/favicon.ico")?
        .map_file("/main.css", "src/styles/main.css")?
        .map_dir("/static", "src/assets/")?;

    let router = Router::new()
        .get("/", routes::get_hello)?
        .get("/hello", routes::get_hello)?
        .get("/send", routes::get_file)?
        .post("/send", routes::post_file)?
        .get("/paste", routes::get_paste)?
        .post("/paste", routes::post_paste)?
        .get("/mirror", routes::get_mirror)?
        .post("/mirror", routes::post_mirror)?
        .get("/slow", routes::get_slow)?
        // 404 Catch all
        .get("/*", routes::get_404)?
        .set_file_server(file_server);

    let server = WebServer::new(HOSTNAME, router)?;
    server.run()
}
