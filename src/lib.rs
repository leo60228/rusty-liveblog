use anyhow::{anyhow, Result};
use autopilot::bitmap::capture_screen;
use autopilot::key;
use image::{FilterType, ImageOutputFormat};
use maplit::*;
use notify_rust::Notification;
use oauth_1a::*;
use pulldown_cmark::{html, Parser};
use std::fs;
use std::io::Cursor;
use std::process::{Command, Stdio};
use tumblr_api::*;
use x11::keysym;

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
        let err = output
            .status
            .code()
            .map(|code| format!("Zenity exited with code {}!", code))
            .unwrap_or_else(|| "Zenity exited with failure!".to_string());

        return Err(anyhow!(err));
    }

    Ok(stdout)
}

#[derive(Clone, Copy)]
pub struct XKeyCode(pub u32);

impl key::KeyCodeConvertible for XKeyCode {
    fn code(&self) -> u64 {
        self.0.into()
    }
}

pub fn play_pause() {
    key::tap(&XKeyCode(keysym::XF86XK_AudioPlay), &[], 150, 150);
}

pub fn screenshot() -> Result<Vec<u8>> {
    let bitmap = capture_screen()?;
    let mut cursor = Cursor::new(Vec::new());

    bitmap
        .image
        .resize(1280, 720, FilterType::Triangle)
        .write_to(&mut cursor, ImageOutputFormat::PNG)?;

    Ok(cursor.into_inner())
}

pub fn info(msg: &str) -> Result<()> {
    Notification::new()
        .summary("rusty-liveblog")
        .body(msg)
        .timeout(notify_rust::Timeout::Milliseconds(2000))
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

pub fn get_consumer_key() -> Result<String> {
    Ok(std::fs::read_to_string(format!(
        "{}/consumer_key.txt",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

pub fn get_consumer_secret() -> Result<String> {
    Ok(std::fs::read_to_string(format!(
        "{}/consumer_secret.txt",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

pub fn authenticate() -> Result<Tumblr> {
    let path = format!("{}/tumblr.toml", env!("CARGO_MANIFEST_DIR"));
    if let Ok(tumblr) = fs::read_to_string(&path)
        .map_err(anyhow::Error::from)
        .and_then(|x| toml::from_str(&x).map_err(From::from))
    {
        Ok(tumblr)
    } else {
        let mut args = std::env::args().skip(1);
        let client_id = ClientId(args.next().ok_or_else(|| anyhow!("Missing client ID!"))?);
        let client_secret = ClientSecret(args.next().ok_or_else(|| anyhow!("Missing client ID!"))?);
        let tumblr = Tumblr::authorize_local(client_id, client_secret)?;
        fs::write(&path, toml::ser::to_string_pretty(&tumblr)?)?;
        Ok(tumblr)
    }
}

pub fn upload(tumblr: &mut Tumblr, png: Vec<u8>, caption: Option<String>) -> Result<()> {
    let mut blocks = vec![];
    blocks.push(PostContent::Image(ImageBlock {
        media: vec![MediaObject {
            file: MediaFile::Identifier("screenshot".to_string()),
            mime_type: Some(mime::IMAGE_PNG),
            width: None,
            height: None,
            original_dimensions_missing: false,
            cropped: false,
            has_original_dimensions: false,
        }],
        ..Default::default()
    }));
    if let Some(caption) = caption {
        blocks.push(PostContent::Text(TextBlock {
            text: caption,
            subtype: None,
        }));
    }
    tumblr.new_post(
        "leo60228.tumblr.com".into(),
        blocks,
        hashmap! {
            "screenshot".to_string() => MediaUpload {
                bytes: png,
                mime_type: mime::IMAGE_PNG,
                filename: "screenshot.png".into(),
            }
        },
    )?;
    Ok(())
}
