use axum::{
    routing::{get, post},
    Router,
};

use crate::{articles::Status, editor::auth_layer::EditorAuthLayer, *};

mod auth_layer;
mod edit_article;

use edit_article::{get_edit_article, post_edit_article, publish_article};

const EDITOR_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/editor/index.html"));
const EDITOR_ARTICLE_SMALL_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/editor/article_small.html"));
const EDITOR_ARTICLE_TEMPLATE: Template<'static> =
    Template::new(include_str!("../../html/editor/edit_article.html"));

pub fn editor_routes(state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/", get(render_editor_articles))
        .route("/article/:uuid", get(get_edit_article))
        .route("/article/:uuid", post(post_edit_article))
        .route("/publish", post(publish_article))
        .layer(EditorAuthLayer::new(state))
}

fn render_article_small(article: &Article) -> String {
    EDITOR_ARTICLE_SMALL_TEMPLATE.render(vec![
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

pub async fn render_editor_articles(
    State(state): State<ServerState>,
    jar: CookieJar,
) -> Html<String> {
    let articles = Article::get_all_articles()
        .iter()
        .filter(|a| a.status == Status::NeedsReview)
        .map(render_article_small)
        .collect::<Vec<_>>()
        .join("<br />");

    render_with_header(
        jar,
        state,
        EDITOR_TEMPLATE
            .render(vec![ArgEntry::new("articles", articles.as_str().into())])
            .as_str()
            .into(),
    )
}
