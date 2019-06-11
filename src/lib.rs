use autopilot::key;
use autopilot::bitmap::capture_screen;
use image::ImageOutputFormat;
use notify_rust::Notification;
use simple_error::SimpleError;
use std::error::Error;
use std::collections::HashMap;
use std::io::Cursor;
use std::process::{Command, Stdio};
use pulldown_cmark::{html, Parser};
use typed_builder::TypedBuilder;
use x11::keysym;
use serde::{Serialize, Deserialize};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn md2html(md: &str) -> String {
    let parser = Parser::new(md);

    let mut html = String::new();
    html::push_html(&mut html, parser);

    let trimmed_len = html.trim_end().len();
    html.truncate(trimmed_len);

    html
}

pub fn multiline_dialog(title: &str) -> Result<String> {
    let output = Command::new("zenity")
        .args(&["--text-info", "--editable", "--width=300", "--height=150"])
        .arg(format!("--title={}", title))
        .stderr(Stdio::inherit())
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    if !output.status.success() {
         let err = output.status.code()
             .map(|code| format!("Zenity exited with code {}!", code))
             .unwrap_or_else(|| "Zenity exited with failure!".to_string());

         return Err(Box::new(SimpleError::new(err)));
    }

    Ok(stdout)
}

#[derive(Clone, Copy)]
pub struct XKeyCode(pub u32);

impl key::KeyCodeConvertible for XKeyCode {
    fn code(&self) -> u64 { self.0.into() }
}

pub fn play_pause() {
    key::tap(&XKeyCode(keysym::XF86XK_AudioPlay), &[], 150);
}

pub fn screenshot() -> Result<Vec<u8>> {
    let bitmap = capture_screen()?;
    let mut cursor = Cursor::new(Vec::new());

    bitmap.image.write_to(&mut cursor, ImageOutputFormat::PNG)?;

    Ok(cursor.into_inner())
}

pub fn info(msg: &str) -> Result<()> {
    Notification::new()
        .summary("rusty-liveblog")
        .body(msg)
        .timeout(notify_rust::Timeout::Milliseconds(1000))
        .show()?;

    Ok(())
}

pub fn error(error: &str) {
    Notification::new()
        .summary("Error")
        .body(error)
        .icon("dialog-error")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
        .expect("Failed to show error notification!");
}

pub fn get_password() -> Result<String> {
    Ok(std::fs::read_to_string(format!("{}/password.txt", env!("CARGO_MANIFEST_DIR")))?)
}

pub fn get_token() -> Result<String> {
    Ok(std::fs::read_to_string(format!("{}/elixire.txt", env!("CARGO_MANIFEST_DIR")))?)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WriteFreelyResponse<T> {
    Ok {
        code: u16,
        data: T,
    },
    Err {
        code: u16,
        error_msg: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ElixireResponse<T> {
    Ok(T),
    Err {
        error: bool,
        message: String,
    },
}

macro_rules! response_result {
    ($T:ty) => {
        impl From<ElixireResponse<$T>> for Result<$T> {
            fn from(resp: ElixireResponse<$T>) -> Result<$T> {
                match resp {
                    ElixireResponse::Ok(data) => Ok(data),
                    ElixireResponse::Err { message, .. } => Err(Box::new(SimpleError::new(message))),
                }
            }
        }

        impl From<WriteFreelyResponse<$T>> for Result<$T> {
            fn from(resp: WriteFreelyResponse<$T>) -> Result<$T> {
                match resp {
                    WriteFreelyResponse::Ok { data, .. } => Ok(data),
                    WriteFreelyResponse::Err { error_msg, .. } => Err(Box::new(SimpleError::new(error_msg))),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub access_token: String,
    pub user: User,
}

response_result!(Login);

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub email: String,
    pub created: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Font {
    Sans,
    Serif,
    Wrap,
    Mono,
    Code,
}

#[derive(Serialize, Deserialize, Debug, TypedBuilder)]
pub struct NewPost {
    pub body: String,
    #[builder(default)]
    pub title: Option<String>,
    #[builder(default)]
    pub font: Option<Font>,
    #[builder(default)]
    pub lang: Option<String>,
    #[builder(default)]
    pub rtl: Option<bool>,
    #[builder(default)]
    pub created: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: String,
    pub slug: Option<String>,
    pub appearance: String,
    pub language: String,
    pub rtl: bool,
    pub created: String,
    pub updated: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub views: usize,
}

response_result!(Post);

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub url: String,
    pub shortname: String,
    pub delete_url: String,
}

response_result!(Image);

pub fn authenticate(username: &str, password: &str) -> Result<Login> {
    let mut params = HashMap::new();
    params.insert("alias", username);
    params.insert("pass", password);

    reqwest::Client::new().post("https://blog.leo60228.space/api/auth/login")
        .json(&params)
        .send()?
        .json::<WriteFreelyResponse<Login>>()?
        .into()
}

pub fn post(token: &str, post: &NewPost) -> Result<Post> {
     reqwest::Client::new().post("https://blog.leo60228.space/api/collections/leo60228/posts")
        .header("Authorization", format!("Token {}", token))
        .json(post)
        .send()?
        .json::<WriteFreelyResponse<Post>>()?
        .into()
}

pub fn upload(token: &str, png: Vec<u8>) -> Result<Image> {
    let part = reqwest::multipart::Part::bytes(png)
        .file_name("image.png")
        .mime_str("image/png")?;
    let form = reqwest::multipart::Form::new()
        .part("f", part);

    reqwest::Client::new().post("https://elixire.leo60228.space/api/upload?admin")
        .multipart(form)
        .header("Authorization", token)
        .send()?
        .json::<ElixireResponse<Image>>()?
        .into()
}
