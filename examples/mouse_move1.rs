use razerctl::{init, mouse_move};
use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    init()?;
    println!("Initialized");

    // Move mouse in a square pattern
    for _ in 0..3 {
        mouse_move(100, 0)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(0, 100)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(-100, 0)?;
        thread::sleep(Duration::from_millis(500));
        mouse_move(0, -100)?;
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
