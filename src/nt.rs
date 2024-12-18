use std::ffi::c_void;
use std::mem::size_of;

use windows::core::{Error, PWSTR};
use windows::Wdk::Foundation::{NtClose, OBJECT_ATTRIBUTES};
use windows::Wdk::Storage::FileSystem::{NtOpenDirectoryObject, NtQueryDirectoryObject};
use windows::Wdk::System::SystemServices::DIRECTORY_QUERY;
use windows::Win32::Foundation::{
    BOOLEAN, HANDLE, STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS, UNICODE_STRING,
};
use windows::Win32::System::Kernel::OBJ_CASE_INSENSITIVE;

#[repr(C)]
struct ObjectDirectoryInformation {
    name: UNICODE_STRING,
    type_name: UNICODE_STRING,
}

pub fn find_sym_link(dir: &str, name: &str) -> Result<String, Error> {
    let dir_handle = match open_directory(None, dir, DIRECTORY_QUERY) {
        Ok(handle) => handle,
        Err(e) => {
            return Err(e);
        }
    };

    let mut query_context: u32 = 0;
    let mut length: u32;

    unsafe {
        loop {
            length = 0;
            let status = NtQueryDirectoryObject(
                dir_handle,
                Some(std::ptr::null_mut()),
                0,
                BOOLEAN(1), // returnsingleentry
                BOOLEAN(0), // restartscan
                &mut query_context,
                Some(&mut length),
            );

            if status != STATUS_BUFFER_TOO_SMALL {
                if status != STATUS_SUCCESS {
                    return Err(Error::from_win32()); // Error querying directory
                }
                break; // No entries found.
            }

            let mut buffer = vec![0u8; length as usize];
            let obj_info = buffer.as_mut_ptr() as *mut ObjectDirectoryInformation;

            let status = NtQueryDirectoryObject(
                dir_handle,
                Some(buffer.as_mut_ptr() as *mut c_void),
                length,
                BOOLEAN(1), // returnsingleentry
                BOOLEAN(0), // restartscan
                &mut query_context,
                Some(&mut length),
            );

            if status != STATUS_SUCCESS {
                return Err(Error::from_win32()); // Error querying directory
            }

            // Extract the name of the object.
            let obj_name = String::from_utf16_lossy(std::slice::from_raw_parts(
                (*obj_info).name.Buffer.0,
                (*obj_info).name.Length as usize / 2,
            ));

            //Check if the object name is the one we are looking for
            if obj_name.contains(name) {
                let _ = NtClose(dir_handle);
                return Ok(obj_name);
            }
        }

        // Close the directory handle
        let _ = NtClose(dir_handle);
    }

    // If no match was found.
    Err(Error::new(
        Error::from_win32().code(),
        format!("Symbolic link '{}' not found in '{}'", name, dir),
    ))
}

fn open_directory(
    root_handle: Option<HANDLE>,
    dir: &str,
    desired_access: u32,
) -> Result<HANDLE, Error> {
    // Convert directory string to UTF-16 for Windows compatibility
    // Add null terminator with chain(Some(0))
    let dir_wide: Vec<u16> = dir.encode_utf16().chain(Some(0)).collect();

    // Create UNICODE_STRING structure required by NT APIs
    // Length is in bytes (2 bytes per UTF-16 character)
    // Subtract 1 from length to exclude null terminator
    let us_dir = UNICODE_STRING {
        Buffer: PWSTR(dir_wide.as_ptr() as *mut _),
        Length: ((dir_wide.len() - 1) * 2) as u16,
        MaximumLength: (dir_wide.len() * 2) as u16,
    };

    // Initialize OBJECT_ATTRIBUTES structure required by NT APIs
    // This structure describes how to open/create the object
    let obj_attr = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: root_handle.unwrap_or_default(),
        ObjectName: &us_dir as *const _,
        Attributes: OBJ_CASE_INSENSITIVE as u32,
        ..Default::default()
    };

    // Initialize handle that will receive the opened directory
    let mut dir_handle = HANDLE::default();

    unsafe {
        // Call NT API to open directory object
        let status = NtOpenDirectoryObject(&mut dir_handle, desired_access, &obj_attr);

        if status == STATUS_SUCCESS {
            Ok(dir_handle)
        } else {
            Err(Error::from_win32()) // Converts NTSTATUS to a Rust Error
        }
    }
}
