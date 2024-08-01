use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Version;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleV0_1_0 {
    pub title: String,
    pub content: String,
    /// author username
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Published,
    NeedsReview,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleV0_2_0 {
    pub title: String,
    pub content: String,
    /// author username
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uuid: Uuid,
    pub status: Status,
}

pub fn convert_article_v0_1_0_to_v0_2_0(article: ArticleV0_1_0) -> ArticleV0_2_0 {
    ArticleV0_2_0 {
        title: article.title,
        content: article.content,
        author: article.author,
        created_at: article.created_at,
        updated_at: article.updated_at,
        uuid: article.uuid,
        status: Status::Published,
    }
}

pub fn read_articles(dir_path: PathBuf) -> Vec<Vec<u8>> {
    let mut data = Vec::new();

    for entry in dir_path
        .read_dir()
        .expect("failed to read articles directory")
    {
        let entry = entry.expect("failed to read entry");
        let article_data = fs::read(entry.path()).expect("failed to read article file");
        data.push(article_data);
    }

    data
}

// TODO: make this actually work dynamically
pub fn convert_articles(v1: Version, v2: Version, articles: Vec<Vec<u8>>) -> Vec<(Uuid, Vec<u8>)> {
    articles
        .into_iter()
        .map(|a| {
            let article = bincode::deserialize::<ArticleV0_1_0>(a.as_slice())
                .expect("failed to deserialize article");
            let converted_article = convert_article_v0_1_0_to_v0_2_0(article);
            (
                converted_article.uuid,
                bincode::serialize(&converted_article).expect("failed to serialize article"),
            )
        })
        .collect()
}

pub fn write_articles(dir_path: PathBuf, articles: Vec<(Uuid, Vec<u8>)>) {
    for (uuid, article) in articles {
        let file_path = dir_path.join(format!("{}.dat", uuid));
        fs::write(file_path, article).expect("failed to write article file");
    }
}
