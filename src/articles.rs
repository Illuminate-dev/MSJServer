use std::time::SystemTime;

use axum::extract::{Path, State};
use uuid::Uuid;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub content: String,
    /// author username
    pub author: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub uuid: Uuid,
}

impl Article {
    pub fn create_new(title: String, content: String, author: String) -> Self {
        let uuid = Uuid::new_v4();
        println!("created new article with uuid: {}", uuid);
        Article {
            title,
            content,
            author,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            uuid,
        }
    }

    fn render_content(&self) -> String {
        self.content.replace('\n', "<br />")
    }
}

pub async fn get_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    println!("getting article with id: {}", id);
    let articles = state.articles.lock().expect("failed to lock mutex");
    if let Some(article) = articles.iter().find(|a| a.uuid == id) {
        let article = article.clone();
        drop(articles);
        render_with_header(
            jar,
            state,
            ARTICLE_PAGE_TEMPLATE
                .render(vec![
                    article.title.as_str().into(),
                    article.render_content().as_str().into(),
                ])
                .as_str()
                .into(),
        )
    } else {
        drop(articles);
        render_with_header(jar, state, INVALID_PAGE_TEMPLATE.into())
    }
}
