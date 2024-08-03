use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Version, VERSION_COUNT};

type ConversionFn = fn(Vec<u8>) -> Vec<u8>;

// [v1->v2,v2->v3, etc]
const UPGRADE_FN: [Option<ConversionFn>; VERSION_COUNT - 1] =
    [Some(convert_article_v0_1_0_to_v0_2_0)];

const GET_UUID_FN: [fn(&[u8]) -> Uuid; VERSION_COUNT] = [get_uuid_v0_1_0, get_uuid_v0_2_0];

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

fn get_uuid_v0_1_0(data: &[u8]) -> Uuid {
    bincode::deserialize::<ArticleV0_1_0>(data)
        .expect("failed to deserailize")
        .uuid
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

fn get_uuid_v0_2_0(data: &[u8]) -> Uuid {
    bincode::deserialize::<ArticleV0_2_0>(data)
        .expect("failed to deserailize")
        .uuid
}

pub fn convert_article_v0_1_0_to_v0_2_0(article: Vec<u8>) -> Vec<u8> {
    let article_v0_1_0: ArticleV0_1_0 =
        bincode::deserialize(article.as_slice()).expect("failed to deserialize");
    bincode::serialize(&ArticleV0_2_0 {
        title: article_v0_1_0.title,
        content: article_v0_1_0.content,
        author: article_v0_1_0.author,
        created_at: article_v0_1_0.created_at,
        updated_at: article_v0_1_0.updated_at,
        uuid: article_v0_1_0.uuid,
        status: Status::Published,
    })
    .expect("failed to serialize")
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

pub fn convert_articles(v1: Version, v2: Version, articles: Vec<Vec<u8>>) -> Vec<(Uuid, Vec<u8>)> {
    articles
        .into_iter()
        .map(|mut a| {
            let mut cur_ver = v1;
            while cur_ver < v2 {
                let f = UPGRADE_FN[u8::from(cur_ver) as usize];
                if let Some(f) = f {
                    a = f(a);
                }
                cur_ver += 1;
            }
            (GET_UUID_FN[u8::from(v2) as usize](&a), a)
        })
        .collect()
}

pub fn write_articles(dir_path: PathBuf, articles: Vec<(Uuid, Vec<u8>)>) {
    for (uuid, article) in articles {
        let file_path = dir_path.join(format!("{}.dat", uuid));
        fs::write(file_path, article).expect("failed to write article file");
    }
}
