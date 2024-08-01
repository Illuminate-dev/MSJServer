pub mod admin;
pub mod articles;
pub mod enter;
pub mod home;
pub mod profile;
pub mod publish;
pub mod template;

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use template::{Arg, ArgEntry, Template};

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
    let logged_in_argentry = ArgEntry::new("logged_in", is_logged_in(&state, &jar).into());

    HEADER_TEMPLATE.render_html(vec![logged_in_argentry, ArgEntry::new("main", to_render)])
}

pub const ACCOUNT_NOT_FOUND_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/account_not_found.html"));
pub const NOT_FOUND_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/404.html"));
pub const NOT_LOGGED_IN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/not_logged_in.html"));
pub const NOT_AUTHOIRZED_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/errors/not_authorized.html"));
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
pub const ADMIN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/admin/index.html"));

pub async fn invalid_page(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Editor,
    User,
}

impl Perms {
    pub fn as_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Admin => "Admin",
            Self::Editor => "Editor",
            Self::User => "User",
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Admin, Self::Editor, Self::User].iter().copied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub permission: Perms,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            permission: Perms::User,
            password_hash: get_sha256(&password),
            created_at: Utc::now(),
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

/// Returns the permission level of the user with the given username or None if the user does not exist.
pub fn get_perms(state: &ServerState, username: &str) -> Option<Perms> {
    let accounts = state.accounts.lock().expect("failed to lock mutex");
    accounts
        .iter()
        .find(|a| a.username == username)
        .map(|a| a.permission)
}
