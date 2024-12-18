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

/// IOCTL code for mouse control operations
const IOCTL_MOUSE: u32 = 0x88883020;

/// Structure for mouse control parameters
#[repr(C)]
#[derive(Copy, Clone)]
struct MouseIoctlStruct {
    field1: i32,
    field2: i32,
    max_val: i32,
    up_down: i32,
    field5: i32,
    x: i32,
    y: i32,
    field8: i32,
}

/// This function attempts to find and open a handle to the Razer device
/// through its symbolic link. It must be called before using any other
/// functions in this library.
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
        if let Ok(name) = nt::find_sym_link("\\GLOBAL??", "RZCONTROL") {
            let sym_link = format!("\\\\?\\{}", name);
            let wide_name: Vec<u16> = sym_link.encode_utf16().chain(Some(0)).collect();

            //Open a handle to the device using the symbolic link.
            let handle = CreateFileW(
                PCWSTR(wide_name.as_ptr()),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE::default(),
            );

            //If the handle is valid, set the global DEVICE variable to the handle, otherwise return an error
            match handle {
                Ok(h) if !h.is_invalid() => {
                    DEVICE = h;
                    Ok(())
                }
                _ => Err(Error::last_os_error()),
            }
        } else {
            //If the symbolic link is not found, return an error
            Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Symbolic link not found",
            ))
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
    let ioctl_struct = MouseIoctlStruct {
        field1: 0,
        field2: 2,
        max_val: if from_start_point { 0 } else { MAX_MOUSE_COORD },
        up_down: 0,
        field5: 0,
        x: x.clamp(-MAX_MOUSE_COORD, MAX_MOUSE_COORD),
        y: y.clamp(-MAX_MOUSE_COORD, MAX_MOUSE_COORD),
        field8: 0,
    };
    send_mouse_ioctl(&ioctl_struct)
}

///Sends a mouse click event
pub fn mouse_click(up_down: i32) -> Result<(), Error> {
    let ioctl_struct = MouseIoctlStruct {
        field1: 0,
        field2: 2,
        max_val: 0,
        up_down,
        field5: 0,
        x: 0,
        y: 0,
        field8: 0,
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

        if let Err(err) = result {
            eprintln!("DeviceIoControl failed: {:?}", err);
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
