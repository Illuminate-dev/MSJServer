use axum::response::{Html, IntoResponse};

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

    pub fn render(&self, args: Vec<String>) -> impl IntoResponse {
        let mut content = self.content.to_string();
        for arg in args {
            content = content.replacen("{}", &arg, 1);
        }
        Html(content)
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

#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl Account {
    pub fn new(username: String, email: String, password: String) -> Self {
        Self {
            username,
            email,
            password,
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
