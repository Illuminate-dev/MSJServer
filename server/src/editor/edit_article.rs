use axum::{extract::Path, response::Redirect, Form};
use uuid::Uuid;

use super::EDITOR_ARTICLE_TEMPLATE;
use crate::{articles::Status, *};

pub async fn get_edit_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(uuid): Path<Uuid>,
) -> Html<String> {
    if let Some(article) = Article::get_article_by_uuid(uuid) {
        render_with_error(state, jar, article, None)
    } else {
        render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into())
    }
}

fn render_with_error(
    state: ServerState,
    jar: CookieJar,
    article: Article,
    error: Option<String>,
) -> Html<String> {
    let article_content = article.content;
    let article_date = article.created_at.format("%B %e, %Y").to_string();

    let title_entry = ArgEntry::new("title", Arg::Text(article.title.as_str()));
    let author_entry = ArgEntry::new("author", Arg::Text(article.author.as_str()));
    let date_entry = ArgEntry::new("date", Arg::Text(article_date.as_str()));
    let content_entry = ArgEntry::new("content", Arg::Text(article_content.as_str()));
    let uuid = article.uuid.to_string();
    let uuid_entry = ArgEntry::new("uuid", Arg::Text(uuid.as_str()));
    let error = error.unwrap_or_default();
    let error = ArgEntry::new("error", Arg::Text(error.as_str()));

    render_with_header(
        jar,
        state,
        EDITOR_ARTICLE_TEMPLATE
            .render(vec![
                title_entry,
                author_entry,
                date_entry,
                content_entry,
                uuid_entry,
                error,
            ])
            .as_str()
            .into(),
    )
}

#[derive(Deserialize)]
pub struct EditArticleForm {
    title: Option<String>,
    content: Option<String>,
    uuid: Uuid,
}

pub async fn post_edit_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    form: Form<EditArticleForm>,
) -> Result<Redirect, Html<String>> {
    if let Some(mut article) = Article::get_article_by_uuid(form.uuid) {
        if let Some((title, content)) = form.title.as_ref().zip(form.content.as_ref()) {
            let title = title.trim().to_string();
            let content = content.trim().to_string();

            article.title = title;
            article.content = content;

            article.write_to_file().expect("failed to save file");

            Ok(Redirect::to(
                format!("/editor/article/{}", form.uuid).as_str(),
            ))
        } else {
            Err(render_with_error(
                state,
                jar,
                article,
                Some("invalid form data".to_string()),
            ))
        }
    } else {
        Err(render_with_header(
            jar,
            state,
            NOT_FOUND_PAGE_TEMPLATE.into(),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct PublishData {
    uuid: Uuid,
}

pub async fn publish_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    Form(data): Form<PublishData>,
) -> Result<Redirect, Html<String>> {
    let mut article = Article::get_article_by_uuid(data.uuid);

    if let Some(article) = article.as_mut() {
        article.status = Status::Published;
        article.write_to_file().expect("failed to save file");

        Ok(Redirect::to(format!("/article/{}", data.uuid).as_str()))
    } else {
        Err(render_with_header(
            jar,
            state,
            NOT_FOUND_PAGE_TEMPLATE.into(),
        ))
    }
}
