use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use backend::Template;
use clap::Parser;
use std::str::FromStr;
use std::{fs, io::Read};
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};

const HEADER_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/header_template.html"));
const INVALID_PAGE_TEMPLATE: Template<'static> = Template::new(include_str!("../html/404.html"));

#[derive(Parser, Debug)]
#[clap(name = "backend", about = "backend for msj website")]
struct Options {
    /// log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// address to bind to
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// port to bind to
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let opts = Options::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", opts.log_level.as_str());
    }

    tracing_subscriber::fmt::init();

    let app = app();

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opts.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opts.port,
    ));

    log::info!("listening on {}", sock_addr);

    axum::serve(
        tokio::net::TcpListener::bind(sock_addr)
            .await
            .expect("failed to bind"),
        app.into_make_service(),
    )
    .await
    .expect("server failed to start")
}

fn app() -> Router {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let asset_service = ServeDir::new(assets_dir);

    let css_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("css");
    let css_service = ServeDir::new(css_dir);
    Router::new()
        .route("/", get(index))
        .fallback(invalid_page)
        .nest_service("/assets", asset_service)
        .nest_service("/css", css_service)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

async fn index() -> impl IntoResponse {
    HEADER_TEMPLATE.render(vec!["Hello, world!".to_string()])
}

async fn invalid_page() -> impl IntoResponse {
    HEADER_TEMPLATE.render(vec![INVALID_PAGE_TEMPLATE.into()])
}
