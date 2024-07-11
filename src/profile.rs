use axum::extract::Query;

use crate::*;

pub fn render_article_small(article: &Article) -> String {
    format!(
        "<a href=\"/article/{}\">
            <div class=\"article-small\">
                <img src=\"{}\" alt=\"placeholder image\"/>
                <div class=\"article-small-info\">
                    <div>
                        <p class=\"article-link\">{}</p>
                        <p class=\"article-desc\">{}</p>
                    </div>
                    <p class=\"article-timestamp\">Created on {}</p>
                </div>
            </div>
        </a>",
        article.uuid,
        "http://via.placeholder.com/640x360",
        article.title,
        article.content.chars().take(100).collect::<String>() + "...",
        article.created_at.format("%B %e, %Y")
    )
}

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
    let articles = Article::get_all_articles();
    let articles_rendered = articles
        .iter()
        .filter(|a| a.author == account_name)
        .map(render_article_small)
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
