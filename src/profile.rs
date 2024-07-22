use axum::extract::Query;

use crate::*;

#[derive(Deserialize)]
pub struct ProfileQuery {
    account_name: Option<String>,
}

pub async fn get_profile(
    State(state): State<ServerState>,
    jar: CookieJar,
    Query(q): Query<ProfileQuery>,
) -> impl IntoResponse {
    if let Some(account_name) = q.account_name {
        render_profile(jar, state, account_name.as_str())
    } else if let Some(account_name) = get_logged_in(&state, &jar) {
        render_profile(jar, state, account_name.as_str())
    } else {
        render_with_header(jar, state, NOT_LOGGED_IN_PAGE_TEMPLATE.into())
    }
}

fn render_profile(jar: CookieJar, state: ServerState, account_name: &str) -> Html<String> {
    let mut articles = Article::get_all_articles();

    articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let articles_rendered = articles
        .iter()
        .filter(|a| a.author == account_name)
        .map(|a| a.render_article_small())
        .collect::<Vec<_>>()
        .join("<br />");

    render_with_header(
        jar,
        state,
        PROFILE_PAGE_TEMPLATE
            .render(vec![
                account_name.into(),
                if articles_rendered.is_empty() {
                    "No articles found".into()
                } else {
                    articles_rendered.as_str().into()
                },
            ])
            .as_str()
            .into(),
    )
}
