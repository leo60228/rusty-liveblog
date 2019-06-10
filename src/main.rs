use rusty_liveblog::*;
use std::thread;
use simple_error::SimpleError;

fn run() -> Result<()> {
    play_pause();

    let screenshot_thread = thread::spawn(|| screenshot().map_err(|err| SimpleError::new(
        format!("Error taking screenshot!: {}", err)
    )));
    let caption = multiline_dialog("Caption")?;
    let screenshot = screenshot_thread.join().map_err(|_| SimpleError::new("Panic taking screenshot!"))??;

    play_pause();

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
