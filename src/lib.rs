mod nt;

use std::{ffi::c_void, io::Error, ptr};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Storage::FileSystem::{
            CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::IO::DeviceIoControl,
    },
};

/// Global device handle
static mut DEVICE: HANDLE = HANDLE(ptr::null_mut());

/// Maximum value for mouse coordinates
const MAX_MOUSE_COORD: i32 = 32767;

/// IOCTL code for communicating with the Razer device
const IOCTL_MOUSE: u32 = 0x88883020;

/// Structure for mouse control parameters
#[repr(C)]
#[derive(Copy, Clone, Default)]
struct MouseIoctlStruct {
    field1: i32,
    field2: i32,
    max_val: i32,
    up_down: i32,
    field5: i32,
    x_coord: i32,
    y_coord: i32,
    field8: i32,
}

/// Sets up the device handle for communication
///
/// # Returns
/// * `Ok(())` - Device initialized successfully
/// * `Err(Error)` - Failed to initialize device
///
/// # Example
/// ```rust
/// fn main() -> std::io::Result<()> {
///     razerctl::init()?;
///     Ok(())
/// }
/// ```
#[allow(static_mut_refs)]
pub fn init() -> Result<(), Error> {
    unsafe {
        //Close the device handle if it already exists
        if !DEVICE.is_invalid() {
            let _ = CloseHandle(DEVICE);
        }

        //Find the symbolic link to the device
        match nt::find_sym_link("\\GLOBAL??", "RZCONTROL") {
            Ok(name) => {
                let sym_link = format!("\\\\?\\{}", name);
                let wide_name: Vec<u16> = sym_link.encode_utf16().chain(Some(0)).collect();

                // Open a handle to the device using the symbolic link.
                let handle = CreateFileW(
                    PCWSTR(wide_name.as_ptr()),
                    0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_FLAGS_AND_ATTRIBUTES(0),
                    HANDLE::default(),
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

/// Moves the mouse cursor to specified coordinates
///
/// # Arguments
/// * `x` - X coordinate (-32767 to 32767)
/// * `y` - Y coordinate (-32767 to 32767)
/// * `from_start_point` - If true, moves relative to the current position
///
/// # Returns
/// * `Ok(())` - Mouse moved successfully
/// * `Err(Error)` - Failed to move mouse
///
/// # Example
/// ```rust
/// fn main() -> std::io::Result<()> {
///     razerctl::init()?;
///     // Move mouse to absolute position (100, 100)
///     razerctl::mouse_move(100, 100, false)?;
///     Ok(())
/// }
/// ```
pub fn mouse_move(x: i32, y: i32, from_start_point: bool) -> Result<(), Error> {
    if !(-MAX_MOUSE_COORD..=MAX_MOUSE_COORD).contains(&x)
        || !(-MAX_MOUSE_COORD..=MAX_MOUSE_COORD).contains(&y)
    {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Coordinates out of range, must be between {} and {}",
                -MAX_MOUSE_COORD, MAX_MOUSE_COORD
            ),
        ));
    }

    let ioctl_struct = MouseIoctlStruct {
        field2: 2,
        max_val: if from_start_point { 0 } else { MAX_MOUSE_COORD },
        x_coord: x,
        y_coord: y,
        ..Default::default()
    };

    send_mouse_ioctl(&ioctl_struct)
}

/// Sends either a left click mouse down
/// 
/// # Arguments
/// * `up_or_down` - If true, sends a left mouse button down event; if false, sends a left mouse button up event
/// 
/// # Returns
/// * `Ok(())` - Mouse click event successfully sent
/// * `Err(Error)` - Failed to send mouse click event
/// 
/// # Example
/// ```rust
/// fn main() -> std::io::Result<()> {
///     razerctl::init()?;
///     // Simulate left mouse button press (down)
///     razerctl::mouse_click(true)?;
///     // Simulate left mouse button release (up)
///     razerctl::mouse_click(false)?;
///     Ok(())
/// }
/// ```
pub fn mouse_click(up_or_down: bool) -> Result<(), Error> {
    let ioctl_struct = MouseIoctlStruct {
        field1: 0,
        field2: 2,
        max_val: 0,
        up_down: if up_or_down { 1 } else { 2 },
        field5: 0,
        x_coord: 0,
        y_coord: 0,
        field8: 1,
    };

    send_mouse_ioctl(&ioctl_struct)
}

/// Internal function to send mouse control commands to the device
///
/// # Arguments
/// * `data` - Mouse control parameters
///
/// # Returns
/// * `Ok(())` - Command sent successfully
/// * `Err(Error)` - Failed to send command
fn send_mouse_ioctl(data: &MouseIoctlStruct) -> Result<(), Error> {
    unsafe {
        let mut bytes_returned = 0u32;

        let result = DeviceIoControl(
            DEVICE,
            IOCTL_MOUSE,
            Some(data as *const _ as *const c_void),
            std::mem::size_of::<MouseIoctlStruct>() as u32,
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
