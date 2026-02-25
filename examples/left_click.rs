use razerctl::{RazerDevice, MouseButton};
use std::io::Error;

fn main() -> Result<(), Error> {
    let mut device = RazerDevice::new()?;
    println!("Initialized");

    send_left_click(&mut device, true);
    std::thread::sleep(std::time::Duration::from_secs(1));
    send_left_click(&mut device, false);

    Ok(())
}

fn send_left_click(device: &mut RazerDevice, up_down: bool) {
    match device.mouse_click(MouseButton::Left, up_down) {
        Ok(_) => {
            println!("Left click sent: {}", up_down);
        }
        Err(e) => {
            eprintln!("Error sending left click: {}", e);
        }
    }
}
