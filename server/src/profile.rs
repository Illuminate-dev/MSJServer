use axum::extract::Path;

use crate::{articles::Status, *};

pub async fn get_profile(
    State(state): State<ServerState>,
    jar: CookieJar,
    path: Option<Path<String>>,
) -> impl IntoResponse {
    if let Some(Path(account_name)) = path {
        if account_name.is_empty() {
            render_self_profile(jar, state)
        } else {
            render_profile(jar, state, account_name.as_str())
        }
    } else {
        render_self_profile(jar, state)
    }
}

fn render_self_profile(jar: CookieJar, state: ServerState) -> Html<String> {
    if let Some(account_name) = get_logged_in(&state, &jar) {
        render_profile(jar, state, account_name.as_str())
    } else {
        render_with_header(jar, state, NOT_LOGGED_IN_PAGE_TEMPLATE.into())
    }
}

fn render_profile(jar: CookieJar, state: ServerState, account_name: &str) -> Html<String> {
    let accounts = state.accounts.lock().expect("failed to lock accounts");
    let account = accounts.iter().find(|a| a.username == account_name);
    if account.is_none() {
        drop(accounts);
        return render_with_header(jar, state, ACCOUNT_NOT_FOUND_PAGE_TEMPLATE.into());
    }
    let account = account.unwrap();

    let account_rank = account.permission.as_string();
    let created_at = account.created_at.format("%B %e, %Y").to_string();
    // let comments_posted = account.created_at.format("%B %e, %Y").to_string();
    // let karma = account.created_at.format("%B %e, %Y").to_string();

    drop(accounts);

    let mut articles = Article::get_all_articles()
        .into_iter()
        .filter(|a| a.status == Status::Published)
        .collect::<Vec<_>>();

    articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let articles_rendered = articles
        .iter()
        .filter(|a| a.author == account_name)
        .map(|a| a.render_article_small())
        .collect::<Vec<_>>();
    let article_count = articles_rendered.len();
    let articles_rendered = articles_rendered.join("<br />");

    render_with_header(
        jar,
        state,
        PROFILE_PAGE_TEMPLATE
            .render(vec![
                ArgEntry::new("username", account_name.into()),
                ArgEntry::new("rank", account_rank.as_str().into()),
                ArgEntry::new("created_at", created_at.as_str().into()),
                ArgEntry::new("article_count", article_count.to_string().as_str().into()),
                ArgEntry::new(
                    "articles",
                    if articles_rendered.is_empty() {
                        "No articles found".into()
                    } else {
                        articles_rendered.as_str().into()
                    },
                ),
            ])
            .as_str()
            .into(),
    )
}
