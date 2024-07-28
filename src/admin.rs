use itertools::Itertools;
use pin_project::pin_project;
use std::{cmp::Ordering, future::Future};

use axum::{
    body::Body,
    extract::{Query, Request},
    http::Response,
    routing::get,
    Router,
};
use tower::{Layer, Service};

use crate::*;

#[derive(Clone)]
struct AdminAuthLayer {
    state: ServerState,
}

impl<S> Layer<S> for AdminAuthLayer {
    type Service = AdminAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AdminAuth {
            state: self.state.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
struct AdminAuth<S> {
    inner: S,
    state: ServerState,
}

impl<S> Service<Request<Body>> for AdminAuth<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, Body>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let jar = CookieJar::from_headers(req.headers());

        if let Some(name) = get_logged_in(&self.state, &jar) {
            if get_perms(&self.state, &name) == Some(Perms::Admin) {
                ResponseFuture::future(self.inner.call(req))
            } else {
                ResponseFuture::error(
                    render_with_header(
                        jar,
                        self.state.clone(),
                        NOT_AUTHOIRZED_PAGE_TEMPLATE.into(),
                    )
                    .into_response(),
                )
            }
        } else {
            ResponseFuture::error(
                render_with_header(jar, self.state.clone(), NOT_AUTHOIRZED_PAGE_TEMPLATE.into())
                    .into_response(),
            )
        }
    }
}
#[pin_project(project = ResponseFutureProj)]
enum ResponseFuture<F, B> {
    Future(#[pin] F),
    Error(Option<Response<B>>),
}

impl<F, B> ResponseFuture<F, B> {
    fn future(f: F) -> Self {
        Self::Future(f)
    }

    fn error(res: Response<B>) -> Self {
        Self::Error(Some(res))
    }
}

impl<F, B, E> Future for ResponseFuture<F, B>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<B>, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project() {
            ResponseFutureProj::Future(f) => f.poll(cx),
            ResponseFutureProj::Error(e) => {
                let res = e.take().expect("polled after ready");
                std::task::Poll::Ready(Ok(res))
            }
        }
    }
}

pub fn admin_routes(state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/", get(get_admin_page))
        .layer(AdminAuthLayer { state })
}

#[derive(Debug, Deserialize)]
pub enum AdminPageQueryType {
    User,
    Comment,
    Article,
}

#[derive(Debug, Deserialize)]
pub struct AdminPageQuery {
    #[serde(rename = "type")]
    query_type: AdminPageQueryType,
    #[serde(rename = "query")]
    term: String,
}

fn render_admin_page(
    jar: CookieJar,
    state: ServerState,
    error: Option<String>,
    panel: Option<String>,
) -> Html<String> {
    render_with_header(
        jar,
        state,
        ADMIN_PAGE_TEMPLATE
            .render(vec![
                ArgEntry::new("error", error.unwrap_or_default().as_str().into()),
                ArgEntry::new("panel", panel.unwrap_or_default().as_str().into()),
            ])
            .as_str()
            .into(),
    )
}

const ADMIN_USER_RESULT_TEMPLATE: Template<'static> =
    Template::new(include_str!("../html/admin/user_result.html"));

fn render_user_result(account: &Account) -> String {
    ADMIN_USER_RESULT_TEMPLATE.render(vec![
        ArgEntry::new("username", account.username.as_str().into()),
        ArgEntry::new("email", account.email.as_str().into()),
        ArgEntry::new("rank", account.permission.as_string().as_str().into()),
        ArgEntry::new(
            "created_at",
            account
                .created_at
                .format("%B %e, %Y")
                .to_string()
                .as_str()
                .into(),
        ),
    ])
}

fn compute_similarity(a: &str, b: &str) -> usize {
    let a = a.to_lowercase();
    let b = b.to_lowercase();

    let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];

    #[allow(clippy::needless_range_loop)]
    for i in 1..=a.len() {
        dp[i][0] = i;
    }

    for i in 1..=b.len() {
        dp[0][i] = i;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            dp[i][j] = *[
                (a.chars().nth(i - 1).unwrap() == b.chars().nth(j - 1).unwrap()) as usize
                    + dp[i - 1][j - 1],
                dp[i - 1][j] + 1,
                dp[i][j - 1] + 1,
            ]
            .iter()
            .min()
            .unwrap();
        }
    }

    dp[a.len()][b.len()]
}

fn render_panel(state: &ServerState, query: AdminPageQuery) -> String {
    match query.query_type {
        AdminPageQueryType::User => {
            let accounts = state.accounts.lock().expect("failed to lock accounts");
            let rendered_accounts = accounts
                .iter()
                .sorted_by(|a, b| {
                    let a_similarity = compute_similarity(&a.username, &query.term);
                    let b_similarity = compute_similarity(&b.username, &query.term);

                    b_similarity.cmp(&a_similarity)
                })
                .take(10)
                .map(render_user_result)
                .collect::<Vec<_>>()
                .join("<br />");

            format!("<div id=\"results\">{}</div>", rendered_accounts)
        }
        _ => todo!(),
    }
}

pub async fn get_admin_page(
    State(state): State<ServerState>,
    jar: CookieJar,
    query: Option<Query<AdminPageQuery>>,
) -> Html<String> {
    if let Some(Query(query)) = query {
        let panel = render_panel(&state, query);
        render_admin_page(jar, state, None, Some(panel))
    } else {
        render_admin_page(jar, state, None, None)
    }
}
