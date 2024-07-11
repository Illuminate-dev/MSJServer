use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect, Response},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use uuid::Uuid;

use crate::*;

#[derive(Deserialize, PartialEq)]
pub enum EnterPageAction {
    #[serde(rename = "signup")]
    SignUp,
    #[serde(rename = "login")]
    LogIn,
    #[serde(rename = "logout")]
    LogOut,
}

#[derive(Deserialize)]
pub struct EnterParams {
    action: EnterPageAction,
}

pub async fn get_enter(
    State(state): State<ServerState>,
    jar: CookieJar,
    query: Option<Query<EnterParams>>,
) -> Response {
    if let Some(Query(q)) = query.as_ref() {
        if q.action == EnterPageAction::LogOut {
            return (process_logout(jar, state), Redirect::to("/")).into_response();
        }
    }

    if is_logged_in(&state, &jar) {
        // kind of uneccessary to check again
        return enter_page(jar, state, ALREADY_LOGGED_IN_PAGE_TEMPLATE, None).into_response();
    }

    if let Some(Query(EnterParams { action: query })) = query {
        match query {
            EnterPageAction::SignUp => {
                enter_page(jar, state, SIGNUP_PAGE_TEMPLATE, None).into_response()
            }
            EnterPageAction::LogIn => {
                enter_page(jar, state, LOGIN_PAGE_TEMPLATE, None).into_response()
            }
            EnterPageAction::LogOut => unreachable!(),
        }
    } else {
        enter_page(jar, state, LOGIN_PAGE_TEMPLATE, None).into_response()
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
    query: Query<EnterParams>,
    jar: CookieJar,
    Form(form): Form<EnterForm>,
) -> Result<(CookieJar, Redirect), (StatusCode, impl IntoResponse)> {
    if is_logged_in(&state, &jar) {
        // already logged in
        return Err((
            StatusCode::PRECONDITION_FAILED,
            enter_page(jar, state, ALREADY_LOGGED_IN_PAGE_TEMPLATE, None),
        ));
    }

    let Query(EnterParams { action: query }) = query;
    match query {
        EnterPageAction::SignUp => create_account(state, form, jar),
        EnterPageAction::LogIn => login_account(state, form, jar),
        _ => Err((
            StatusCode::BAD_REQUEST,
            enter_page(jar, state, LOGIN_PAGE_TEMPLATE, Some("no action specified")),
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

fn process_logout(jar: CookieJar, state: ServerState) -> CookieJar {
    let mut sessions = state.sessions.lock().expect("failed to lock mutex");

    sessions.retain(|s| {
        s.id != jar
            .get(SESSION_COOKIE_NAME)
            .map(|c| c.value())
            .unwrap_or("")
    });

    drop(sessions);
    jar.remove(Cookie::from(SESSION_COOKIE_NAME))
}
