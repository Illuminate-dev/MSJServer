use anyhow::Result;
use axum::extract::{Path, State};
use chrono::{offset::Utc, DateTime};
use uuid::Uuid;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub content: String,
    /// author username
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
            uuid,
        }
    }

    fn render_content(&self) -> String {
        self.content.replace('\n', "<br />")
    }

    pub fn from_file(file_path: PathBuf) -> Result<Self> {
        let data = fs::read(file_path)?;
        bincode::deserialize(data.as_slice()).map_err(Into::into)
    }

    pub fn write_to_file(&self) -> Result<()> {
        let data = bincode::serialize(self)?;
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("articles")
            .join(format!("{}.dat", self.uuid));
        fs::write(file_path, data)?;
        Ok(())
    }

    pub fn get_all_articles() -> Vec<Self> {
        let mut articles = Vec::new();
        let articles_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("articles");
        for entry in articles_dir
            .read_dir()
            .expect("failed to read articles directory")
        {
            let entry = entry.expect("failed to read entry");
            let file_path = entry.path();
            articles.push(Self::from_file(file_path).expect("failed to read article from file"));
        }
        articles
    }
}

pub async fn get_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    println!("getting article with id: {}", id);

    let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("articles")
        .join(format!("{}.dat", id));

    if file_path.exists() {
        let article = Article::from_file(file_path).expect("failed to read article from file");
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
        render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into())
    }
}
