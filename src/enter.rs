use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use uuid::Uuid;

use crate::*;

#[derive(Deserialize)]
pub struct EnterPageQuery {
    #[serde(default)]
    signup: Option<bool>,
    #[serde(default)]
    login: Option<bool>,
}

pub async fn get_enter(
    State(state): State<ServerState>,
    jar: CookieJar,
    query: Option<Query<EnterPageQuery>>,
) -> Html<String> {
    let sessions = state.sessions.lock().expect("failed to lock mutex");

    if jar.get(SESSION_COOKIE_NAME).is_some()
        && sessions
            .iter()
            .any(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value())
    {
        drop(sessions);
        // kind of uneccessary to check again
        return enter_page(jar, state, ALREADY_LOGGED_IN_PAGE_TEMPLATE, None);
    }

    drop(sessions);

    if let Some(query) = query {
        match (query.signup, query.login) {
            (Some(true), _) => enter_page(jar, state, SIGNUP_PAGE_TEMPLATE, None),
            _ => enter_page(jar, state, LOGIN_PAGE_TEMPLATE, None),
        }
    } else {
        enter_page(jar, state, LOGIN_PAGE_TEMPLATE, None)
    }
}

fn enter_page(
    jar: CookieJar,
    state: ServerState,
    page: Template,
    error: Option<&str>,
) -> Html<String> {
    render_with_header(
        jar,
        state,
        (page.render(vec![error.unwrap_or("").into()]))
            .as_str()
            .into(),
    )
}

#[derive(Deserialize)]
pub struct EnterForm {
    username: Option<String>,
    email: String,
    password: String,
}

pub async fn post_enter(
    State(state): State<ServerState>,
    query: Query<EnterPageQuery>,
    jar: CookieJar,
    Form(form): Form<EnterForm>,
) -> Result<(CookieJar, Redirect), (StatusCode, impl IntoResponse)> {
    let sessions = state.sessions.lock().expect("failed to lock mutex");
    if jar.get(SESSION_COOKIE_NAME).is_some()
        && sessions
            .iter()
            .any(|s| s.id == jar.get(SESSION_COOKIE_NAME).unwrap().value())
    {
        drop(sessions);
        // already logged in
        return Err((
            StatusCode::PRECONDITION_FAILED,
            enter_page(jar, state, ALREADY_LOGGED_IN_PAGE_TEMPLATE, None),
        ));
    }

    drop(sessions);

    match (query.signup, query.login) {
        (Some(true), Some(true)) => Err((
            StatusCode::BAD_REQUEST,
            enter_page(
                jar,
                state,
                SIGNUP_PAGE_TEMPLATE,
                Some("Cannot sign up and log in at the same time"),
            ),
        )),
        (Some(true), _) => create_account(state, form, jar),
        (_, Some(true)) => login_account(state, form, jar),
        _ => Err((
            StatusCode::BAD_REQUEST,
            enter_page(jar, state, LOGIN_PAGE_TEMPLATE, Some("No action specified")),
        )),
    }
}

fn create_account(
    state: ServerState,
    form: EnterForm,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), (StatusCode, Html<String>)> {
    if form.username.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            enter_page(
                jar,
                state,
                SIGNUP_PAGE_TEMPLATE,
                Some("No username specified"),
            ),
        ));
    }

    let mut accounts = state.accounts.lock().expect("failed to lock mutex");
    if accounts
        .iter()
        .any(|a| &a.username == form.username.as_ref().unwrap() || a.email == form.email)
    {
        drop(accounts);
        return Err((
            StatusCode::BAD_REQUEST,
            enter_page(
                jar,
                state,
                SIGNUP_PAGE_TEMPLATE,
                Some("Account with that username/email already exists!"),
            ),
        ));
    }

    accounts.push(Account::new(
        form.username.as_ref().unwrap().clone(),
        form.email,
        form.password,
    ));

    drop(accounts);

    state.write_accounts().expect("failed to write accounts");

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
) -> Result<(CookieJar, Redirect), (StatusCode, Html<String>)> {
    let accounts = state.accounts.lock().expect("failed to lock mutex");
    if let Some(account) = accounts
        .iter()
        .find(|a| a.email == form.email && a.password_hash == get_sha256(&form.password))
    {
        let mut sessions = state.sessions.lock().expect("failed to lock mutex");

        let id = Uuid::new_v4().to_string();

        sessions.push(Session::new(id.clone(), account.username.clone()));

        Ok((
            jar.add(Cookie::new(SESSION_COOKIE_NAME, id)),
            Redirect::to("/"),
        ))
    } else {
        drop(accounts);
        Err((
            StatusCode::UNAUTHORIZED,
            enter_page(
                jar,
                state,
                LOGIN_PAGE_TEMPLATE,
                Some("Invalid email or password"),
            ),
        ))
    }
}
