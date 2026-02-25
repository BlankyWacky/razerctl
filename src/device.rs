use std::{ffi::c_void, io::Error};
use crate::types::*;
use crate::utils;
use crate::key_translation;
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

/// IOCTL code for communicating with the Razer device
const RAZER_IOCTL_CODE: u32 = 0x88883020;

/// Represents a connection to the Razer device.
/// 
/// The handle to the device is automatically closed when this struct goes out of scope.
#[derive(Debug)]
pub struct RazerDevice {
    handle: HANDLE,
}

// Safety: The Windows HANDLE is owned exclusively by RazerDevice.
// Operations using the handle (DeviceIoControl) are thread-safe at the OS level
// as long as we don't close it while another thread is using it.
// `&mut self` requirement for methods ensures we don't have data races.
unsafe impl Send for RazerDevice {}
unsafe impl Sync for RazerDevice {}

impl RazerDevice {
    /// Initializes a new connection to the Razer device.
    ///
    /// This function:
    /// 1. Locates the Razer device's symbolic link.
    /// 2. Opens a handle to the device for communication.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the symbolic link cannot be found or the device handle cannot be opened.
    pub fn new() -> Result<Self, Error> {
        let handle = Self::open_device_handle()?;
        Ok(RazerDevice { handle })
    }

    /// Internal helper to find and open the device handle.
    fn open_device_handle() -> Result<HANDLE, Error> {
        // Locate the Razer device
        let name = utils::find_sym_link("\\GLOBAL??", "RZCONTROL").map_err(|e| {
            Error::new(
                e.kind(),
                format!("Error while trying to find symbolic link: {}", e),
            )
        })?;

        let sym_link = format!("\\\\?\\{}", name);
        let wide_name: Vec<u16> = sym_link.encode_utf16().chain(Some(0)).collect();

        unsafe {
            let handle = CreateFileW(
                PCWSTR(wide_name.as_ptr()),
                GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            );

            // Check if handle is valid
            match handle {
                Ok(h) if !h.is_invalid() => Ok(h),
                _ => Err(Error::last_os_error()),
            }
        }
    }

    /// Attempts to re-open the device handle.
    fn reconnect(&mut self) -> Result<(), Error> {
        if !self.handle.is_invalid() {
            unsafe { CloseHandle(self.handle).ok() };
        }
        self.handle = Self::open_device_handle()?;
        Ok(())
    }

    /// Send a relative mouse move event with the specified coordinates.
    pub fn mouse_move(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let mut control = RzControl::new(Type::Mouse);
        unsafe {
            let mouse_data = control.mouse_data_mut();
            mouse_data.x = x;
            mouse_data.y = y;
            mouse_data.absolute_coord = 0;
            mouse_data.movement = 1;
            mouse_data.button_flags = MouseButtons::new();
        }
        self.send_control(&control)
    }

    /// Send a mouse button click event.
    pub fn mouse_click(&mut self, button: MouseButton, down: bool) -> Result<(), Error> {
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

        self.send_control(&control)
    }

    /// Sends a key down (press) event.
    pub fn key_down(&mut self, vk: u8) -> Result<(), Error> {
        self.send_keyboard_input(vk, true)
    }

    /// Sends a key up (release) event.
    pub fn key_up(&mut self, vk: u8) -> Result<(), Error> {
        self.send_keyboard_input(vk, false)
    }

    fn send_keyboard_input(&mut self, vk: u8, is_down: bool) -> Result<(), Error> {
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
            keyboard_data.flags = if is_down { KEY_MAKE } else { KEY_BREAK };
            if key_translation::is_extended_key(vk) {
                keyboard_data.flags |= KEY_E0;
            }
            keyboard_data.unit_id = 0;
            keyboard_data.reserved = 0;
            keyboard_data.extra_information = 0;
        }

        self.send_control(&control)
    }

    fn send_control(&mut self, data: &RzControl) -> Result<(), Error> {
        let mut bytes_returned = 0u32;
        
        let result = unsafe {
            DeviceIoControl(
                self.handle,
                RAZER_IOCTL_CODE,
                Some(data as *const _ as *const c_void),
                std::mem::size_of::<RzControl>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )
        };

        if result.is_err() {
            // Attempt to reinitialize device
            self.reconnect().map_err(|e| Error::new(std::io::ErrorKind::Other, format!("Reconnection failed: {}", e)))?;
            
            unsafe {
                DeviceIoControl(
                    self.handle,
                    RAZER_IOCTL_CODE,
                    Some(data as *const _ as *const c_void),
                    std::mem::size_of::<RzControl>() as u32,
                    None,
                    0,
                    Some(&mut bytes_returned),
                    None,
                ).map_err(Error::from)?;
            }
        }
        Ok(())
    }
}

impl Drop for RazerDevice {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                CloseHandle(self.handle).ok();
            }
        }
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
