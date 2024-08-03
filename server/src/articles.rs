use anyhow::Result;
use axum::extract::{Path, State};
use chrono::{offset::Utc, DateTime};
use uuid::Uuid;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Published,
    NeedsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub content: String,
    /// author username
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uuid: Uuid,
    pub status: Status,
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
            status: Status::NeedsReview,
        }
    }

    fn article_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("articles")
    }

    pub fn from_file(file_path: PathBuf) -> Result<Self> {
        let data = fs::read(file_path)?;
        bincode::deserialize(data.as_slice()).map_err(Into::into)
    }

    pub fn write_to_file(&mut self) -> Result<()> {
        self.updated_at = Utc::now();
        let data = bincode::serialize(self)?;
        let file_path = Self::article_dir().join(format!("{}.dat", self.uuid));
        fs::write(file_path, data)?;
        Ok(())
    }

    pub fn delete(self) -> Result<()> {
        let file_path = Self::article_dir().join(format!("{}.dat", self.uuid));

        fs::remove_file(file_path)?;
        Ok(())
    }

    pub fn get_all_articles() -> Vec<Self> {
        let mut articles = Vec::new();
        for entry in Self::article_dir()
            .read_dir()
            .expect("failed to read articles directory")
        {
            let entry = entry.expect("failed to read entry");
            let file_path = entry.path();
            articles.push(Self::from_file(file_path).expect("failed to read article from file"));
        }
        articles
    }

    pub fn get_article_by_uuid(uuid: Uuid) -> Option<Self> {
        let file_path = Self::article_dir().join(format!("{}.dat", uuid));
        if file_path.exists() {
            Some(Self::from_file(file_path).expect("failed to read article from file"))
        } else {
            None
        }
    }

    pub fn render_article_small(&self) -> String {
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
            self.uuid,
            "http://via.placeholder.com/640x360",
            self.title,
            self.author,
            Self::format_for_description(&self.content),
            self.created_at.format("%B %e, %Y")
        )
    }

    pub fn format_for_description(s: &str) -> String {
        let mut result = String::new();
        let mut iter = s.chars().take(100);

        while let Some(char) = iter.next() {
            match char {
                // '\\' => {
                //     iter.next();
                // }
                '<' => {
                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(char) = iter.next() {
                        // if char == '\\' {
                        //     iter.next();
                        // }
                        if char == '>' {
                            break;
                        }
                    }

                    // result.push_str("<br />");
                }
                _ => {
                    result.push(char);
                }
            }
        }

        result + "..."
    }
}

pub async fn get_article(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    println!("getting article with id: {}", id);

    if let Some(article) = Article::get_article_by_uuid(id) {
        if article.status != Status::Published {
            return render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into());
        }

        let article_content = article.content;
        let article_date = article.updated_at.format("%B %e, %Y").to_string();

        let title_entry = ArgEntry::new("title", Arg::Text(article.title.as_str()));
        let author_entry = ArgEntry::new("author", Arg::Text(article.author.as_str()));
        let date_entry = ArgEntry::new("date", Arg::Text(article_date.as_str()));
        let content_entry = ArgEntry::new("content", Arg::Text(article_content.as_str()));

        render_with_header(
            jar,
            state,
            ARTICLE_PAGE_TEMPLATE
                .render(vec![title_entry, author_entry, date_entry, content_entry])
                .as_str()
                .into(),
        )
    } else {
        render_with_header(jar, state, NOT_FOUND_PAGE_TEMPLATE.into())
    }
}
