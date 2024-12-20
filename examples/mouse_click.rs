use razerctl::{init, mouse_click};
use std::io::Error;

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

    print_send_mouse_click(true);
    std::thread::sleep(std::time::Duration::from_secs(1));
    print_send_mouse_click(false);

    Ok(())
}

fn print_send_mouse_click(up_down: bool) {
    match mouse_click(up_down) {
        Ok(_) => {
            println!("Mouse click sent: {}", up_down);
        }
        Err(e) => {
            eprintln!("Error sending mouse click: {}", e);
        }
    }
}
