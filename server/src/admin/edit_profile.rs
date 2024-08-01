use axum::{extract::Path, response::Redirect, Form};

use crate::{admin::ADMIN_EDIT_PROFILE_TEMPLATE, *};

fn admin_render_article_small(article: &Article) -> String {
    format!(
        "<a href=\"/admin/article/{}\">
          <div class=\"article-small\">
              <img src=\"{}\" alt=\"placeholder image\"/>
              <div class=\"article-small-info\">
                  <div>
                      <p class=\"article-title\">{}</p>
                      <p class=\"article-author\">By: {}</p>
                      <p class=\"article-desc\">{}</p>
                  </div>
                  <div class=\"bottom-wrapper\">
                      <p class=\"article-timestamp\">Created on {}</p>
                  </div>
              </div>
          </div>
        </a>",
        article.uuid,
        "http://via.placeholder.com/640x360",
        article.title,
        article.author,
        Article::format_for_description(&article.content),
        article.created_at.format("%B %e, %Y"),
    )
}

pub async fn get_edit_profile(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(username): Path<String>,
) -> Html<String> {
    render_with_error(state, jar, username, None)
}

fn render_with_error(
    state: ServerState,
    jar: CookieJar,
    username: String,
    error: Option<String>,
) -> Html<String> {
    let accounts = state.accounts.lock().expect("failed to lock accounts");
    let account = accounts.iter().find(|a| a.username == username);
    if account.is_none() {
        drop(accounts);
        return render_with_header(jar, state, ACCOUNT_NOT_FOUND_PAGE_TEMPLATE.into());
    }
    let account = account.unwrap();

    let account_rank = account.permission.as_string();
    let created_at = account.created_at.format("%B %e, %Y").to_string();
    let email = account.email.to_owned();
    // let comments_posted = account.created_at.format("%B %e, %Y").to_string();
    // let karma = account.created_at.format("%B %e, %Y").to_string();

    // determine which rank is selected
    let selector_strings = Perms::iter()
        .map(|perm| perm.as_str().to_lowercase() + "_selected")
        .collect::<Vec<_>>();
    let selectors = Perms::iter()
        .zip(selector_strings.iter())
        .map(|(perm, s)| ArgEntry::new(s.as_str(), (account.permission == perm).into()))
        .collect::<Vec<_>>();

    drop(accounts);

    let mut articles = Article::get_all_articles();

    articles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let articles_rendered = articles
        .iter()
        .filter(|a| a.author == username)
        .map(admin_render_article_small)
        .collect::<Vec<_>>();
    let article_count = articles_rendered.len();
    let articles_rendered = articles_rendered.join("<br />");

    render_with_header(
        jar,
        state,
        ADMIN_EDIT_PROFILE_TEMPLATE
            .render(
                vec![
                    ArgEntry::new("username", username.as_str().into()),
                    ArgEntry::new("email", email.as_str().into()),
                    ArgEntry::new("rank", account_rank.as_str().into()),
                    ArgEntry::new("created_at", created_at.as_str().into()),
                    ArgEntry::new("article_count", article_count.to_string().as_str().into()),
                    ArgEntry::new(
                        "articles",
                        if articles_rendered.is_empty() {
                            "No articles found".into()
                        } else {
                            articles_rendered.as_str().into()
                        },
                    ),
                    ArgEntry::new("error", error.unwrap_or_default().as_str().into()),
                ]
                .into_iter()
                .chain(selectors)
                .collect(),
            )
            .as_str()
            .into(),
    )
}

#[derive(Deserialize)]
pub enum EditProfileAction {
    Delete,
    Edit,
}

#[derive(Deserialize)]
pub struct EditProfileForm {
    action: EditProfileAction,
    rank: Option<Perms>,
}

pub async fn post_edit_profile(
    State(state): State<ServerState>,
    jar: CookieJar,
    Path(username): Path<String>,
    Form(form): Form<EditProfileForm>,
) -> Result<Redirect, Html<String>> {
    let accounts = state.accounts.lock().expect("failed to lock mutex");

    if let Some(account) = accounts.iter().find(|a| a.username == username) {
        if account.permission == Perms::Admin {
            drop(accounts);
            return Err(render_with_error(
                state,
                jar,
                username,
                Some("Cannot edit admin account".into()),
            ));
        }
        drop(accounts);

        match form.action {
            EditProfileAction::Delete => {
                let mut accounts = state.accounts.lock().expect("failed to lock mutex");
                accounts.retain(|a| a.username != username);
                drop(accounts);
                Ok(Redirect::to("/admin"))
            }
            EditProfileAction::Edit => {
                if let Some(perm) = form.rank {
                    let mut accounts = state.accounts.lock().expect("failed to lock mutex");
                    if let Some(account) = accounts.iter_mut().find(|a| a.username == username) {
                        account.permission = perm;
                        drop(accounts);
                    } else {
                        drop(accounts);
                        return Err(render_with_error(
                            state,
                            jar,
                            username,
                            Some("Account not found".into()),
                        ));
                    }
                }
                Ok(Redirect::to("/admin"))
            }
        }
    } else {
        drop(accounts);
        Err(render_with_error(
            state,
            jar,
            username,
            Some("Account not found".into()),
        ))
    }
}
