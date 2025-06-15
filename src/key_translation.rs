/// Maps a USB HID Usage ID to a Razer-specific scan code (MakeCode).
/// The tables are extracted from the kbddef.dat file in Razer Synapse.
pub fn usage_id_to_make_code(usage_id: u16) -> i16 {
    // This table was found to have 149 elements. The check and type have been updated to reflect this.
    // By using 'const', we ensure the table is stored in the program's static data, not on the stack.
    if (usage_id as usize) < USAGE_ID_TABLE.len() {
        return USAGE_ID_TABLE[usage_id as usize];
    } else if (181..=183).contains(&usage_id) {
        let table: [i16; 3] = [ 25, 16, 36 ];
        return table[(usage_id - 181) as usize];
    } else if usage_id == 205 {
        return 34;
    } else if (224..=234).contains(&usage_id) {
        let table: [i16; 11] = [ 29, 42, 56, 91, 29, 54, 56, 92, -2, 48, 46 ];
        return table[(usage_id - 224) as usize];
    }
    -1
}

const USAGE_ID_TABLE: [i16; 149] = [
    -1, 255, 252, -3, 30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49,
    24, 25, 16, 19, 31, 20, 22, 47, 17, 45, 21, 44, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 28, 1, 14, 15, 57, 12, 13, 26, 27, 43, 43, 39, 40, 41, 51, 52, 53,
    58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88, 55, 70, 29, 82, 71, 73,
    83, 79, 81, 77, 75, 80, 72, 69, 53, 55, 74, 78, 28, 79, 80, 81, 75, 76, 77,
    71, 72, 73, 82, 83, 86, 93, 94, 89, 100, 101, 102, 103, 104, 105, 106, 107,
    108, 109, 110, 118, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3,
    94, 95, 99, -3, 126, -3, 115, 112, 125, 121, 123, 92, -3, -3, -3, 242,
    241, 120, 119, 118
];


/// Maps a Windows Virtual-Key (VK) code to a USB HID Usage ID.
/// This is a standard mapping. A complete list is available online.
/// Returns 0 if no mapping found.
pub fn vk_to_usage_id(vk: u8) -> u16 {
    // This is a partial list. For a full implementation, you'd need all VK codes.
    // See Microsoft's documentation or online resources for a complete table.
    match vk as u16 {
        0x08 => 42, // Backspace
        0x09 => 43, // Tab
        0x0D => 40, // Enter
        0x10 => 225, // Shift (Left)
        0x11 => 224, // Control (Left)
        0x12 => 226, // Alt (Left)
        0x13 => 72, // Pause
        0x14 => 57, // Caps Lock
        0x1B => 41, // Escape
        0x20 => 44, // Space
        0x21 => 75, // Page Up
        0x22 => 78, // Page Down
        0x23 => 77, // End
        0x24 => 74, // Home
        0x25 => 80, // Left Arrow
        0x26 => 82, // Up Arrow
        0x27 => 79, // Right Arrow
        0x28 => 81, // Down Arrow
        0x2D => 76, // Insert
        0x2E => 73, // Delete
        // 0-9
        0x30..=0x39 => (vk - 0x30) as u16 + 30 - 1,
        // A-Z
        0x41..=0x5A => (vk - 0x41) as u16 + 4,
        0x5B => 227, // Left Win
        0x5C => 231, // Right Win
        // F1-F12
        0x70..=0x7B => (vk - 0x70) as u16 + 58,
        _ => 0, // No mapping
    }
}

/// Checks if a VK code corresponds to an extended key (requiring the KEY_E0 flag).
pub fn is_extended_key(vk: u8) -> bool {
    matches!(vk as u16,
        0x21..=0x28 // PageUp/Down, End, Home, Arrows
        | 0x2D // Insert
        | 0x2E // Delete
        | 0x5B // Left Win
        | 0x5C // Right Win
    )
}