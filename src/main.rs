use rusty_liveblog::*;
use std::thread;
use std::time::Duration;
use finally_block::finally;

fn run() -> Result<()> {
    thread::sleep(Duration::from_millis(500));
    play_pause();

    let _unpause = finally(|| {
        thread::sleep(Duration::from_millis(500));
        play_pause();
    });

    let screenshot = screenshot()?;
    let caption = multiline_dialog("Caption")?;

    let image = upload(&get_token()?, screenshot)?;

    let login = authenticate("leo60228", &get_password()?)?;
    let new_post = NewPost::builder()
        .body(format!("![Screenshot]({})\n\n{}", image.url, caption))
        .build();
    post(&login.access_token, &new_post)?;

    info("Posted!")?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        let text = err.to_string();
        eprintln!("{}", text);
        error(&text);
        std::process::exit(1);
    }
}
