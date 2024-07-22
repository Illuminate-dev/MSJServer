use crate::*;

pub async fn index(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    let mut articles = Article::get_all_articles();

    articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let articles_rendered = articles
        .iter()
        .map(|a| a.render_article_small())
        .collect::<Vec<_>>()
        .join("<br />");

    render_with_header(
        jar,
        state,
        INDEX_PAGE_TEMPLATE
            .render(vec![articles_rendered.as_str().into()])
            .as_str()
            .into(),
    )
}
