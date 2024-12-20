use razerctl::{init, mouse_move};
use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    //Initialize with default settings
    match init() {
        Ok(_) => {
            println!("Initialized");
        }
        Err(e) => {
            eprintln!("Error initializing: {}", e);
            return Err(e);
        }
    }

    //Move mouse in a square pattern
    for _ in 0..3 {
        mouse_move(100, 0, true)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(0, 100, true)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(-100, 0, true)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(0, -100, true)?;
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
