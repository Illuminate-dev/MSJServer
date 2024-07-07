use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use backend::*;
use clap::Parser;
use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

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

#[derive(Default, Clone)]
struct ServerState {
    sessions: Arc<Mutex<Vec<Session>>>,
    accounts: Arc<Mutex<Vec<Account>>>,
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
        .with_state(ServerState::default())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

async fn index() -> impl IntoResponse {
    HEADER_TEMPLATE.render(vec!["Hello, world!".to_string()])
}

async fn invalid_page() -> impl IntoResponse {
    HEADER_TEMPLATE.render(vec![INVALID_PAGE_TEMPLATE.into()])
}

#[derive(Deserialize)]
struct EnterPageQuery {
    #[serde(default)]
    signup: Option<bool>,
    #[serde(default)]
    login: Option<bool>,
}

async fn get_enter(
    State(state): State<ServerState>,
    jar: CookieJar,
    query: Option<Query<EnterPageQuery>>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().expect("failed to lock mutex");

    if jar.get(SESSION_COOKIE_NAME).is_some()
        && sessions
            .iter()
            .any(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value())
    {
        return HEADER_TEMPLATE.render(vec![ALREADY_LOGGED_IN_PAGE_TEMPLATE.into()]);
    }

    if let Some(query) = query {
        match (query.signup, query.login) {
            (Some(true), _) => HEADER_TEMPLATE.render(vec![SIGNUP_PAGE_TEMPLATE.into()]),
            _ => HEADER_TEMPLATE.render(vec![LOGIN_PAGE_TEMPLATE.into()]),
        }
    } else {
        HEADER_TEMPLATE.render(vec![LOGIN_PAGE_TEMPLATE.into()])
    }
}

#[derive(Deserialize)]
struct EnterForm {
    username: Option<String>,
    email: String,
    password: String,
}

async fn post_enter(
    State(state): State<ServerState>,
    query: Query<EnterPageQuery>,
    jar: CookieJar,
    Form(form): Form<EnterForm>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let sessions = state.sessions.lock().expect("failed to lock mutex");
    if jar.get(SESSION_COOKIE_NAME).is_some()
        && sessions
            .iter()
            .any(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value())
    {
        // already logged in
        return Err(StatusCode::PRECONDITION_FAILED);
    }

    drop(sessions);

    match (query.signup, query.login) {
        (Some(true), Some(true)) => Err(StatusCode::BAD_REQUEST),
        (Some(true), _) => create_account(state, form, jar),
        (_, Some(true)) => login_account(state, form, jar),
        _ => {
            println!("test");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

fn create_account(
    state: ServerState,
    form: EnterForm,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    if form.username.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut accounts = state.accounts.lock().expect("failed to lock mutex");
    if accounts
        .iter()
        .any(|a| &a.username == form.username.as_ref().unwrap() || a.email == form.email)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    accounts.push(Account::new(
        form.username.as_ref().unwrap().clone(),
        form.email,
        form.password,
    ));

    let mut sessions = state.sessions.lock().expect("failed to lock mutex");

    let id = Uuid::new_v4().to_string();

    sessions.push(Session::new(id.clone(), form.username.unwrap()));

    Ok((
        jar.add(Cookie::new(SESSION_COOKIE_NAME, id)),
        Redirect::to("/"),
    ))
}

fn login_account(
    state: ServerState,
    form: EnterForm,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let accounts = state.accounts.lock().expect("failed to lock mutex");
    if let Some(account) = accounts
        .iter()
        .find(|a| a.email == form.email && a.password == form.password)
    {
        let mut sessions = state.sessions.lock().expect("failed to lock mutex");

        let id = Uuid::new_v4().to_string();

        sessions.push(Session::new(id.clone(), account.username.clone()));

        Ok((
            jar.add(Cookie::new(SESSION_COOKIE_NAME, id)),
            Redirect::to("/"),
        ))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
