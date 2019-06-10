use rusty_liveblog::*;
use std::thread;
use std::time::Duration;
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
    thread::sleep(Duration::from_millis(500));
    play_pause();
    let run = run();
    thread::sleep(Duration::from_millis(500));
    play_pause();

    if let Err(err) = run {
        let text = err.to_string();
        eprintln!("{}", text);
        error(&text);
        std::process::exit(1);
    }
}
