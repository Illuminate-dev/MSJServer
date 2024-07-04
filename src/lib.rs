use std::path::PathBuf;

use axum::response::{Html, IntoResponse};

pub struct Template<'a> {
    content: &'a str,
}

impl<'a> Template<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn render(&self, args: Vec<String>) -> impl IntoResponse {
        let mut content = self.content.to_string();
        for arg in args {
            content = content.replacen("{}", &arg, 1);
        }
        Html(content)
    }
}

// for nesting templates
impl<'a> From<Template<'a>> for String {
    fn from(template: Template) -> Self {
        template.content.to_string()
    }
}

impl<'a> From<Template<'a>> for &'a str {
    fn from(template: Template<'a>) -> Self {
        template.content
    }
}
