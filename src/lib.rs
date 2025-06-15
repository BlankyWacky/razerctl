use std::{ffi::c_void, io::Error, ptr};
use types::*;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, GENERIC_WRITE, HANDLE},
        Storage::FileSystem::{
            CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::IO::DeviceIoControl,
    },
};

mod key_translation;
mod types;
mod utils;

/// Global device handle
static mut DEVICE: HANDLE = HANDLE(ptr::null_mut());

/// IOCTL code for communicating with the Razer device
const RAZER_IOCTL_CODE: u32 = 0x88883020;

/// Initializes the connection to the Razer device.
///
/// This function:
/// 1. Closes any existing device handle.
/// 2. Locates the Razer device's symbolic link.
/// 3. Opens a handle to the device for communication.
///
/// # Errors
///
/// Returns an `Error` if the device handle cannot be opened or if the symbolic link cannot be found.
#[allow(static_mut_refs)]
pub fn init() -> Result<(), Error> {
    unsafe {
        // Clean up existing handle if present
        if !DEVICE.is_invalid() {
            CloseHandle(DEVICE).ok();
        }

        // Locate and connect to the Razer device
        match utils::find_sym_link("\\GLOBAL??", "RZCONTROL") {
            Ok(name) => {
                let sym_link = format!("\\\\?\\{}", name);
                let wide_name: Vec<u16> = sym_link.encode_utf16().chain(Some(0)).collect();

                let handle = CreateFileW(
                    PCWSTR(wide_name.as_ptr()),
                    GENERIC_WRITE.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_FLAGS_AND_ATTRIBUTES(0),
                    None,
                );

                // If the handle is valid, set the global DEVICE variable to the handle, otherwise return an error
                match handle {
                    Ok(h) if !h.is_invalid() => {
                        DEVICE = h;
                        Ok(())
                    }
                    _ => Err(Error::last_os_error()),
                }
            }
            Err(e) => Err(Error::new(
                e.kind(),
                format!("Error while trying to find symbolic link, {}", e),
            )),
        }
    }
}

/// Send a relative mouse move event with the specified coordinates.
///
/// # Arguments
///
/// * `x` - The x-coordinate of the mouse movement.
/// * `y` - The y-coordinate of the mouse movement.
///
/// # Errors
///
/// Returns an `Error` if the control command cannot be sent to the device.
pub fn mouse_move(x: i32, y: i32) -> Result<(), Error> {
    let mut control = RzControl::new(Type::Mouse);
    unsafe {
        let mouse_data = control.mouse_data_mut();

        mouse_data.x = x;
        mouse_data.y = y;
        mouse_data.absolute_coord = 0; //TODO: Not working as expected
        mouse_data.movement = 1; // Movement flag, set to 1 to indicate a movement
        mouse_data.button_flags = MouseButtons::new(); // Initialize button states

        send_control(&control)
    }
}

/// Represents mouse buttons.
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Send a mouse button click event.
///
/// # Arguments
///
/// * `button` - The mouse button to click.
/// * `down` - `true` if the button is pressed down, `false` if the button is released.
///
/// # Errors
///
/// Returns an `Error` if the control command cannot be sent to the device.
pub fn mouse_click(button: MouseButton, down: bool) -> Result<(), Error> {
    let mut control = RzControl::new(Type::Mouse);

    unsafe {
        let mouse_data = control.mouse_data_mut();

        mouse_data.button_flags = MouseButtons::new();

        match (button, down) {
            (MouseButton::Left, true) => mouse_data.button_flags.set_flag(L_BUTTON_DOWN, true),
            (MouseButton::Left, false) => mouse_data.button_flags.set_flag(L_BUTTON_UP, true),
            (MouseButton::Right, true) => mouse_data.button_flags.set_flag(R_BUTTON_DOWN, true),
            (MouseButton::Right, false) => mouse_data.button_flags.set_flag(R_BUTTON_UP, true),
            (MouseButton::Middle, true) => mouse_data.button_flags.set_flag(M_BUTTON_DOWN, true),
            (MouseButton::Middle, false) => mouse_data.button_flags.set_flag(M_BUTTON_UP, true),
            (MouseButton::X1, true) => mouse_data.button_flags.set_flag(X_BUTTON1_DOWN, true),
            (MouseButton::X1, false) => mouse_data.button_flags.set_flag(X_BUTTON1_UP, true),
            (MouseButton::X2, true) => mouse_data.button_flags.set_flag(X_BUTTON2_DOWN, true),
            (MouseButton::X2, false) => mouse_data.button_flags.set_flag(X_BUTTON2_UP, true),
        }
    }

    send_control(&control)
}

/// Sends a key down (press) event.
///
/// # Arguments
///
/// * `vk` - The virtual key code of the key to press.
///
/// # Errors
///
/// Returns an `Error` if the key is unknown or the command fails.
pub fn key_down(vk: u8) -> Result<(), Error> {
    send_keyboard_input(vk, true)
}

/// Sends a key up (release) event.
///
/// # Arguments
///
/// * `vk` - The virtual key code of the key to release.
///
/// # Errors
///
/// Returns an `Error` if the key is unknown or the command fails.
pub fn key_up(vk: u8) -> Result<(), Error> {
    send_keyboard_input(vk, false)
}

/// Internal function to send a keyboard input event with translated scan codes.
fn send_keyboard_input(vk: u8, is_down: bool) -> Result<(), Error> {
    let usage_id = key_translation::vk_to_usage_id(vk);
    if usage_id == 0 {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Unknown virtual key code"));
    }

    let make_code = key_translation::usage_id_to_make_code(usage_id);
    if make_code < 0 {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Failed to map key to scan code"));
    }

    let mut control = RzControl::new(Type::Keyboard);
    unsafe {
        let keyboard_data = control.keyboard_data_mut();

        keyboard_data.make_code = make_code as u16;

        // Set flags based on key state and if it's an extended key
        keyboard_data.flags = if is_down { types::KEY_MAKE } else { types::KEY_BREAK };
        if key_translation::is_extended_key(vk) {
            keyboard_data.flags |= types::KEY_E0;
        }

        // Set other fields to 0
        keyboard_data.unit_id = 0;
        keyboard_data.reserved = 0;
        keyboard_data.extra_information = 0;

        send_control(&control)
    }
}

/// Internal function to send commands to the device.
///
/// # Arguments
///
/// * `data` - The control data to send.
///
/// # Errors
///
/// Returns an `Error` if the control command cannot be sent to the device or if the device needs to be reinitialized.
fn send_control(data: &RzControl) -> Result<(), Error> {
    unsafe {
        let mut bytes_returned = 0u32;

        let result = DeviceIoControl(
            DEVICE,
            RAZER_IOCTL_CODE,
            Some(data as *const _ as *const c_void),
            std::mem::size_of::<RzControl>() as u32,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        );

        if let Err(_err) = result {
            // Attempt to reinitialize device
            if let Err(init_err) = init() {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to reinitialize device: {:?}", init_err),
                ));
            }
        }
    }
    Ok(())
}
