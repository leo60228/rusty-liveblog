use rusty_liveblog::*;
use std::thread;
use simple_error::SimpleError;

fn run() -> Result<()> {
    let screenshot_thread = thread::spawn(|| screenshot().map_err(|err| SimpleError::new(
        format!("Error taking screenshot!: {}", err)
    )));
    let caption = multiline_dialog("Caption")?;
    let screenshot = screenshot_thread.join().map_err(|_| SimpleError::new("Panic taking screenshot!"))??;

    Ok(())
}

fn main() {
    play_pause();
    let run = run();
    play_pause();

    if let Err(err) = run {
        let text = err.to_string();
        eprintln!("{}", text);
        error(&text);
        std::process::exit(1);
    }
}
