use std::path::PathBuf;

use axum::response::{Html, IntoResponse};

pub struct Template {
    path: PathBuf,
}

impl Template {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn render(&self, args: Vec<String>) -> impl IntoResponse {
        let mut content =
            std::fs::read_to_string(&self.path).expect("failed to read template file");
        for arg in args {
            content = content.replacen("{}", &arg, 1);
        }
        Html(content)
    }
}
