pub mod enter;

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::response::{Html, IntoResponse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Default, Clone)]
pub struct ServerState {
    sessions: Arc<Mutex<Vec<Session>>>,
    accounts: Arc<Mutex<Vec<Account>>>,
}

impl ServerState {
    pub fn new() -> Self {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        fs::create_dir_all(data_dir.clone()).expect("failed to create data directory");
        let accounts_file = data_dir.join("accounts.dat");
        if accounts_file.exists() {
            let accounts = bincode::deserialize(
                fs::read(accounts_file)
                    .expect("failed to read accounts file")
                    .as_slice(),
            )
            .expect("failed to deserialize accounts file");
            Self {
                accounts: Arc::new(Mutex::new(accounts)),
                ..Default::default()
            }
        } else {
            fs::write(
                accounts_file,
                bincode::serialize::<Vec<Account>>(&vec![]).expect("failed to write accounts file"),
            )
            .expect("failed to write accounts file");
            Self::default()
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
}

pub const SESSION_COOKIE_NAME: &str = "msj_session";

pub const HEADER_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/header_template.html"));
pub const INVALID_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/404.html"));
pub const LOGIN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/login.html"));
pub const SIGNUP_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/signup.html"));
pub const ALREADY_LOGGED_IN_PAGE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/enter/already_logged_in.html"));

pub struct Template<'a> {
    content: &'a str,
}

impl<'a> Template<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn render(&self, args: Vec<String>) -> String {
        let mut content = self.content.to_string();
        for arg in args {
            content = content.replacen("{}", &arg, 1);
        }
        content.replace("{}", "")
    }

    pub fn render_html(&self, args: Vec<String>) -> Html<String> {
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
pub struct Account {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl Account {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            password_hash: get_sha256(&password),
        }
    }
}

pub struct Session {
    pub id: String,
    pub account_username: String,
}

impl Session {
    pub fn new(id: String, account_username: String) -> Self {
        Self {
            id,
            account_username,
        }
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
