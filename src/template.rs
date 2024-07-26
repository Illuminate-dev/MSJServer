use axum::response::Html;

pub struct ArgEntry<'a> {
    key: &'a str,
    value: Arg<'a>,
}

pub enum Arg<'a> {
    Text(&'a str),
    Bool(bool),
}

impl ArgEntry<'_> {
    pub fn new<'a>(key: &'a str, value: Arg<'a>) -> ArgEntry<'a> {
        ArgEntry { key, value }
    }
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(text: &'a str) -> Self {
        Self::Text(text)
    }
}

impl<'a> From<bool> for Arg<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<'a> From<Template<'a>> for Arg<'a> {
    fn from(temp: Template<'a>) -> Self {
        Self::Text(temp.into())
    }
}

pub struct Template<'a> {
    content: &'a str,
}

impl<'a> Template<'a> {
    pub const fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn render(&self, args: Vec<ArgEntry>) -> String {
        let mut content = self.content.to_string();
        for argentry in args {
            match argentry.value {
                Arg::Text(text) => {
                    // format looks weird but the double brackets are escaped to single brackets
                    content = content.replace(format!("{{{}}}", argentry.key).as_str(), text)
                }
                Arg::Bool(value) => {
                    let start_match = format!("{{?{}", argentry.key);
                    let start_length = start_match.len();

                    while let Some(start) = content.find(start_match.as_str()) {
                        let middle = start
                            + start_length
                            + content[start + start_length..]
                                .find('|')
                                .expect("failed to find middle of bool expression");
                        let end = middle
                            + content[middle..]
                                .find('}')
                                .expect("failed to find end of bool expression");

                        let first_opt = String::from(&content[start + start_length..middle]);
                        let second_opt = String::from(content[middle + 1..end].trim());
                        content.replace_range(
                            start..end + 1,
                            if value { &first_opt } else { &second_opt },
                        );
                    }
                }
            }
        }
        content.replace("{}", "")
    }

    pub fn render_html(&self, args: Vec<ArgEntry>) -> Html<String> {
        Html(self.render(args))
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
