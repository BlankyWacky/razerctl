use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    razerctl::init()?;
    println!("Initialized");

    // Space button virtual key code
    const VK_SPACE: u8 = 0x20;

    // Press the Spacebar 5 times with a delay
    for i in 1..=5 {
        println!("Simulating Spacebar press {}...", i);
        razerctl::key_down(VK_SPACE)?;
        thread::sleep(Duration::from_millis(50));
        razerctl::key_up(VK_SPACE)?;
        println!("Spacebar press {} sent.", i);

        // Wait before next press (skip delay after last press)
        if i < 5 {
            thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(())
}
