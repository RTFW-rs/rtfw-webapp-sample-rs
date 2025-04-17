use log::LevelFilter;
use rtfw_http::{router::Router, web_server::WebServer};

mod routes;
mod utils;

const HOSTNAME: &str = "127.0.0.1:7878";

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .init();

    let router = Router::new()
        .get("/", routes::get_hello)?
        .get("/hello", routes::get_hello)?
        .get("/send", routes::get_file)?
        .post("/send", routes::post_file)?
        .get("/paste", routes::get_paste)?
        .get("/paste-data", routes::get_paste_data)?
        .post("/paste", routes::post_paste)?
        .get("/get-mirror", routes::get_mirror)?
        .post("/post-mirror", routes::post_mirror)?
        .get("/slow", routes::get_slow)?
        // 404 Catch all
        .get("/*", routes::get_404)?;

    let server = WebServer::new(HOSTNAME, router)?;
    server.run()
}
