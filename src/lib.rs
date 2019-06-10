use autopilot::key;
use autopilot::bitmap::capture_screen;
use image::ImageOutputFormat;
use notify_rust::Notification;
use simple_error::SimpleError;
use std::error::Error;
use std::io::Cursor;
use std::process::{Command, Stdio};
use pulldown_cmark::{html, Parser};
use x11::keysym;

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

pub fn error(error: &str) {
    Notification::new()
        .summary("Error")
        .body(error)
        .icon("dialog-error")
        .show()
        .expect("Failed to show error notification!");
}
