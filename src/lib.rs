pub mod articles;
pub mod enter;
pub mod home;
pub mod profile;
pub mod publish;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::articles::Article;

pub const SESSION_COOKIE_NAME: &str = "msj_session";

#[derive(Default, Clone)]
pub struct ServerState {
    sessions: Arc<Mutex<Vec<Session>>>,
    accounts: Arc<Mutex<Vec<Account>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self::load_articles_dir();
        let state = Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
            accounts: Arc::new(Mutex::new(Self::read_accounts())),
        };
        let arc_sessions = Arc::clone(&state.sessions);

        thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_secs(60));
            let mut sessions = arc_sessions.lock().expect("failed to lock mutex");
            sessions.retain(|s| !s.is_expired());
        });

        state
    }

    fn read_accounts() -> Vec<Account> {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        fs::create_dir_all(data_dir.clone()).expect("failed to create data directory");
        let accounts_file = data_dir.join("accounts.dat");
        if accounts_file.exists() {
            bincode::deserialize(
                fs::read(accounts_file)
                    .expect("failed to read accounts file")
                    .as_slice(),
            )
            .expect("failed to deserialize accounts file")
        } else {
            fs::write(
                accounts_file,
                bincode::serialize::<Vec<Account>>(&vec![]).expect("failed to write accounts file"),
            )
            .expect("failed to write accounts file");
            Vec::new()
        }
    }

    pub fn write_accounts(&self) -> Result<(), std::io::Error> {
        // accounts file should exist

        let accounts_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("accounts.dat");
        fs::write(
            accounts_file,
            bincode::serialize(
                self.accounts
                    .lock()
                    .expect("failed to lock mutex")
                    .as_slice(),
            )
            .expect("failed to serialize accounts"),
        )
    }

    fn load_articles_dir() {
        let articles_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("articles");
        fs::create_dir_all(articles_dir.clone()).expect("failed to create articles directory");
    }
}

pub const HEADER_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/header_template.html"));

pub fn render_with_header(jar: CookieJar, state: ServerState, to_render: Arg) -> Html<String> {
    if is_logged_in(&state, &jar) {
        HEADER_TEMPLATE.render_html(vec![true.into(), true.into(), to_render])
    } else {
        HEADER_TEMPLATE.render_html(vec![false.into(), false.into(), to_render])
    }
}

pub const NOT_FOUND_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/404.html"));
pub const NOT_LOGGED_IN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/not_logged_in.html"));
pub const INDEX_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/index.html"));
pub const LOGIN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/login.html"));
pub const SIGNUP_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/signup.html"));
pub const ALREADY_LOGGED_IN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/already_logged_in.html"));
pub const PUBLISH_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/publish.html"));
pub const ARTICLE_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/article.html"));
pub const PROFILE_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/profile.html"));

pub async fn invalid_page(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into())
}

pub enum Arg<'a> {
    Text(&'a str),
    Bool(bool),
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(text: &'a str) -> Self {
        Self::Text(text)
    }
}

impl<'a> From<bool> for Arg<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<'a> From<Template<'a>> for Arg<'a> {
    fn from(temp: Template<'a>) -> Self {
        Self::Text(temp.into())
    }
}

pub struct Template<'a> {
    content: &'a str,
}

impl<'a> Template<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn render(&self, args: Vec<Arg>) -> String {
        let mut content = self.content.to_string();
        for arg in args {
            match arg {
                Arg::Text(text) => content = content.replacen("{}", text, 1),
                Arg::Bool(value) => {
                    let start = content
                        .find('{')
                        .expect("failed to find start of bool expression");
                    let middle = start
                        + content[start..]
                            .find('|')
                            .expect("failed to find middle of bool expression");
                    let end = middle
                        + content[middle..]
                            .find('}')
                            .expect("failed to find middle of bool expression");

                    let first_opt = String::from(&content[start + 1..middle]);
                    let second_opt = String::from(content[middle + 1..end].trim());
                    content.replace_range(
                        start..end + 1,
                        if value { &first_opt } else { &second_opt },
                    );
                }
            }
        }
        content.replace("{}", "")
    }

    pub fn render_html(&self, args: Vec<Arg>) -> Html<String> {
        Html(self.render(args))
    }
}

// for nesting templates
impl<'a> From<Template<'a>> for String {
    fn from(template: Template) -> Self {
        template.content.to_string()
    }
}

impl<'a> From<Template<'a>> for &'a str {
    fn from(template: Template<'a>) -> Self {
        template.content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Editor,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub permission: Perms,
    pub email: String,
    pub password_hash: String,
}

impl Account {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            permission: Perms::User,
            password_hash: get_sha256(&password),
        }
    }
}

pub struct Session {
    pub id: String,
    pub account_username: String,
    last_used: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String, account_username: String) -> Self {
        Self {
            id,
            account_username,
            last_used: Utc::now(),
        }
    }

    pub fn extend(&mut self) {
        self.last_used = Utc::now();
    }

    fn is_expired(&self) -> bool {
        self.last_used < Utc::now() - TimeDelta::minutes(30)
    }
}

pub fn get_sha256(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let password_hash = hasher.finalize();
    password_hash.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&format!("{:02x}", byte));
        acc
    })
}

pub fn is_logged_in(state: &ServerState, jar: &CookieJar) -> bool {
    if jar.get(SESSION_COOKIE_NAME).is_none() {
        return false;
    }
    let mut locked_sessions = state.sessions.lock().expect("failed to lock mutex");
    let session = locked_sessions
        .iter_mut()
        .find(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value() && !s.is_expired());

    if let Some(session) = session {
        session.extend();
        true
    } else {
        false
    }
}

pub fn get_logged_in(state: &ServerState, jar: &CookieJar) -> Option<String> {
    jar.get(SESSION_COOKIE_NAME)?;

    let mut locked_sessions = state.sessions.lock().expect("failed to lock mutex");
    let session = locked_sessions
        .iter_mut()
        .find(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value() && !s.is_expired());

    session.map(|s| {
        s.extend();
        s.account_username.clone()
    })
}
