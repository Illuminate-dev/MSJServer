use crate::*;

fn render_article_small(article: &Article) -> String {
    format!(
        "<a href=\"/article/{}\">
            <div class=\"article-small\">
                <img src=\"{}\" alt=\"placeholder image\"/>
                <div class=\"article-small-info\">
                    <div>
                        <p class=\"article-title\">{}</p>
                        <p class=\"article-author\">By: {}</p>
                        <p class=\"article-desc\">{}</p>
                    </div>
                    <p class=\"article-timestamp\">Created on {}</p>
                </div>
            </div>
        </a>",
        article.uuid,
        "http://via.placeholder.com/640x360",
        article.title,
        article.author,
        article.content.chars().take(100).collect::<String>() + "...",
        article.created_at.format("%B %e, %Y")
    )
}

pub async fn index(State(state): State<ServerState>, jar: CookieJar) -> Html<String> {
    let mut articles = Article::get_all_articles();

    articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let articles_rendered = articles
        .iter()
        .map(render_article_small)
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
