use razerctl::{init, mouse_click, MouseButton};
use std::io::Error;

fn main() -> Result<(), Error> {
    init()?;
    println!("Initialized");

    send_left_click(true);
    std::thread::sleep(std::time::Duration::from_secs(1));
    send_left_click(false);

    Ok(())
}

fn send_left_click(up_down: bool) {
    match mouse_click(MouseButton::Left, up_down) {
        Ok(_) => {
            println!("Left click sent: {}", up_down);
        }
        Err(e) => {
            eprintln!("Error sending left click: {}", e);
        }
    }
}
