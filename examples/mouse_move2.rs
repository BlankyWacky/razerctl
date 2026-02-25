use razerctl::RazerDevice;
use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    let mut device = RazerDevice::new()?;
    println!("Initialized");

    // Draw circles with the mouse for 3 seconds, then exit
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        circle(&mut device)?;
    }

    Ok(())
}

/*
Move mouse in a circular pattern with radius 5
We should always sleep for a minimum of 1ms between mouse moves to avoid overloading the system
*/
fn circle(device: &mut RazerDevice) -> Result<(), Error> {
    for i in 0..360 {
        let x = (i as f64).to_radians().cos() * 5.0;
        let y = (i as f64).to_radians().sin() * 5.0;
        device.mouse_move(x as i32, y as i32)?;
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
