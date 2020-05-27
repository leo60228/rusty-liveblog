use anyhow::Result;
use finally_block::finally;
use rusty_liveblog::*;
use std::thread;
use std::time::Duration;

fn run() -> Result<()> {
    thread::sleep(Duration::from_millis(500));
    play_pause();

    let _unpause = finally(|| {
        thread::sleep(Duration::from_millis(500));
        play_pause();
    });

    let screenshot = screenshot()?;
    let caption = multiline_dialog("Caption")?;
    let mut tumblr = authenticate()?;

    upload(&mut tumblr, screenshot, Some(caption))?;

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
