use axum::{extract::State, response::Redirect, Form};

use crate::*;

fn publish_page(jar: CookieJar, state: ServerState, error: Option<&str>) -> impl IntoResponse {
    render_with_header(
        jar,
        state,
        PUBLISH_PAGE_TEMPLATE
            .render(vec![error.unwrap_or("").into()])
            .as_str()
            .into(),
    )
}

pub async fn get_publish(State(state): State<ServerState>, jar: CookieJar) -> impl IntoResponse {
    publish_page(jar, state, None)
}

#[derive(Deserialize)]
pub struct PublishForm {
    title: String,
    content: String,
}

pub async fn post_publish(
    State(state): State<ServerState>,
    jar: CookieJar,
    Form(form): Form<PublishForm>,
) -> Result<Redirect, impl IntoResponse> {
    let title = form.title.trim().to_string();
    let content = form.content.trim().to_string();

    if let Some(account_name) = get_logged_in(&state, &jar) {
        let article = Article::create_new(title, content, account_name);
        article.write_to_file().expect("failed to save file");

        Ok(Redirect::to("/"))
    } else {
        Err(publish_page(
            jar,
            state,
            Some("You must be logged in to publish a post."),
        ))
    }
}
