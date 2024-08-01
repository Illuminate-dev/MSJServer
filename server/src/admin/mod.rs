mod auth_layer;
mod edit_article;
mod edit_profile;

use itertools::Itertools;

use axum::{
    extract::Query,
    routing::{get, post},
    Router,
};

use crate::{
    admin::{
        auth_layer::AdminAuthLayer,
        edit_article::{get_edit_article, post_edit_article},
        edit_profile::{get_edit_profile, post_edit_profile},
    },
    *,
};

const ADMIN_USER_RESULT_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/admin/user_result.html"));
const ADMIN_ARTICLE_RESULT_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/admin/article_result.html"));
const ADMIN_EDIT_PROFILE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/admin/edit_profile.html"));
const ADMIN_EDIT_ARTICLE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/admin/edit_article.html"));

pub fn admin_routes(state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/", get(get_admin_page))
        .route("/profile/:username", get(get_edit_profile))
        .route("/profile/:username", post(post_edit_profile))
        .route("/article/:uuid", get(get_edit_article))
        .route("/article/:uuid", post(post_edit_article))
        .layer(AdminAuthLayer::new(state))
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

fn render_article_result(article: &Article) -> String {
    ADMIN_ARTICLE_RESULT_TEMPLATE.render(vec![
        ArgEntry::new("uuid", article.uuid.to_string().as_str().into()),
        ArgEntry::new("title", article.title.as_str().into()),
        ArgEntry::new("author", article.author.as_str().into()),
        ArgEntry::new(
            "date",
            article
                .created_at
                .format("%B %e, %Y")
                .to_string()
                .as_str()
                .into(),
        ),
        ArgEntry::new(
            "description",
            Article::format_for_description(&article.content)
                .as_str()
                .into(),
        ),
        ArgEntry::new("img_src", "http://via.placeholder.com/640x360".into()),
    ])
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
        AdminPageQueryType::Article => {
            let articles = Article::get_all_articles();

            let rendered_articles = articles
                .iter()
                .sorted_by(|a, b| {
                    let a_similarity = compute_similarity(&a.title, &query.term);
                    let b_similarity = compute_similarity(&b.title, &query.term);

                    b_similarity.cmp(&a_similarity)
                })
                .take(10)
                .map(render_article_result)
                .collect::<Vec<_>>()
                .join("<br />");

            format!("<div id=\"results\">{}</div>", rendered_articles)
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
