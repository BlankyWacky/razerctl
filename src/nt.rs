use std::ffi::c_void;
use std::io::Error;
use std::mem::size_of;

use windows::core::PWSTR;
use windows::Wdk::Foundation::{NtClose, OBJECT_ATTRIBUTES};
use windows::Wdk::Storage::FileSystem::{NtOpenDirectoryObject, NtQueryDirectoryObject};
use windows::Wdk::System::SystemServices::DIRECTORY_QUERY;
use windows::Win32::Foundation::{
    BOOLEAN, HANDLE, NTSTATUS, STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS, UNICODE_STRING,
};
use windows::Win32::System::Kernel::OBJ_CASE_INSENSITIVE;

#[repr(C)]
struct ObjectDirectoryInformation {
    name: UNICODE_STRING,
    type_name: UNICODE_STRING,
}

// Finds a symbolic link in the specified directory by name.
pub fn find_sym_link(dir: &str, name: &str) -> Result<String, Error> {
    // Open the directory for querying symbolic links.
    let dir_handle = open_directory(None, dir, DIRECTORY_QUERY)?;

    let mut query_context: u32 = 0; // Used to track the state of the directory query.
    let mut length: u32;

    unsafe {
        loop {
            length = 0;

            // Query the directory to check if there is enough space to store entries.
            let status = NtQueryDirectoryObject(
                dir_handle,
                Some(std::ptr::null_mut()), // No buffer initially, just checking space.
                0,                          // No data to fetch yet.
                BOOLEAN(1),                 // Request a single entry.
                BOOLEAN(0),                 // Don't restart the scan.
                &mut query_context,         // Context to continue the search.
                Some(&mut length),          // Length of the buffer to be filled.
            );

            // If the buffer size is too small, allocate a new buffer with the correct size.
            if status != STATUS_BUFFER_TOO_SMALL {
                if status != STATUS_SUCCESS {
                    // Return error if status is not success or buffer too small.
                    return Err(status_to_error(status, "unexpected error occurred:"));
                }
                break; // If successful and no data found, exit the loop.
            }

            // Create a buffer large enough to hold the data and extract the object info.
            let mut buffer = vec![0u8; length as usize];
            let obj_info = buffer.as_mut_ptr() as *mut ObjectDirectoryInformation;

            // Re-run the query to fill the buffer with directory entry data.
            let status = NtQueryDirectoryObject(
                dir_handle,
                Some(buffer.as_mut_ptr() as *mut c_void),
                length,
                BOOLEAN(1),         // Request a single entry.
                BOOLEAN(0),         // Don't restart the scan.
                &mut query_context, // Context to continue the search.
                Some(&mut length),  // Length of the filled data.
            );

            if status != STATUS_SUCCESS {
                // Handle errors during query and return the respective error.
                return Err(status_to_error(status, "unexpected error occurred:"));
            }

            // Extract the symbolic link name from the buffer.
            let obj_name = String::from_utf16_lossy(std::slice::from_raw_parts(
                (*obj_info).name.Buffer.0,
                (*obj_info).name.Length as usize / 2, // UTF-16 characters are 2 bytes long.
            ));

            // Check if the object name matches the one we're looking for.
            if obj_name.contains(name) {
                let _ = NtClose(dir_handle); // Close the directory handle after use.
                return Ok(obj_name); // Return the matched object name.
            }
        }

        // Close the directory handle after the search is complete.
        let _ = NtClose(dir_handle);
    }

    // If no matching symbolic link name is found, return a NotFound error.
    Err(Error::new(
        std::io::ErrorKind::NotFound,
        format!("symbolic link name '{}' not found in '{}'", name, dir),
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
            Err(status_to_error(status, "Error opening directory:"))
        }
    }
}

// Convert NTSTATUS to Error
fn status_to_error(status: NTSTATUS, message: &str) -> Error {
    let hresult = status.to_hresult();
    let message = format!("{} {}", message, hresult.message());
    Error::new(std::io::ErrorKind::Other, message)
}
