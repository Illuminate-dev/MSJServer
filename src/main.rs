use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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

    let app = Router::new()
        .route("/", get(hello))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

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

async fn hello() -> impl IntoResponse {
    "hello, world!"
}
