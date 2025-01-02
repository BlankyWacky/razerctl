use razerctl::{init, mouse_move};
use std::{io::Error, thread, time::Duration};

fn main() -> Result<(), Error> {
    //Initialize with default settings
    match init() {
        Ok(_) => {
            println!("Initialized");
        }
        Err(e) => {
            eprintln!("Error initializing: {:?}", e);
            return Err(e);
        }
    }

    //Draw circles with the mouse for 3 seconds, then exit
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        circle()?;
    }

    Ok(())
}

/*
Move mouse in a circular pattern with radius 5
We should always sleep for a minimum of 1ms between mouse moves to avoid overloading the system
*/
fn circle() -> Result<(), Error> {
    for i in 0..360 {
        let x = (i as f64).to_radians().cos() * 5.0;
        let y = (i as f64).to_radians().sin() * 5.0;
        mouse_move(x as i32, y as i32)?;
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
