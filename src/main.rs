use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;
use backend::{
    enter::{get_enter, post_enter},
    *,
};
use clap::Parser;
use std::str::FromStr;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};

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
        .route("/enter", get(get_enter))
        .route("/enter", post(post_enter))
        .fallback(invalid_page)
        .nest_service("/assets", asset_service)
        .nest_service("/css", css_service)
        .with_state(ServerState::new())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

async fn index(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    render_with_header(jar, state, "Hello, world!".into())
}

async fn invalid_page(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    render_with_header(jar, state, INVALID_PAGE_TEMPLATE.into())
}
