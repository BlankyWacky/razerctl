use razerctl::RazerDevice;
use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    let mut device = RazerDevice::new()?;
    println!("Initialized");

    // Move mouse in a square pattern
    for _ in 0..3 {
        device.mouse_move(100, 0)?;
        thread::sleep(Duration::from_millis(500));
        device.mouse_move(0, 100)?;
        thread::sleep(Duration::from_millis(500));
        device.mouse_move(-100, 0)?;
        thread::sleep(Duration::from_millis(500));
        device.mouse_move(0, -100)?;
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
