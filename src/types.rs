#[repr(C)]
pub struct RzControl {
    pub unk1: u32,
    pub r#type: Type,
    pub data: InputData,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum Type {
    Keyboard = 1,
    Mouse = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union InputData {
    pub mouse: MouseInputData,
    pub keyboard: KeyboardInputData,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MouseInputData {
    pub absolute_coord: u32,
    pub button_flags: MouseButtons,
    pub movement: i16,
    pub unk1: u32,
    pub x: i32,
    pub y: i32,
    pub unk2: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KeyboardInputData {
    pub unit_id: u16,       // Unused, can be 0
    pub make_code: u16,     // The hardware scan code for the key
    pub flags: u16,         // KEY_MAKE, KEY_BREAK, KEY_E0, KEY_E1
    pub reserved: u16,      // Must be 0
    pub extra_information: u32, // Unused, can be 0
}

// Flags for KeyboardInputData.flags
pub const KEY_MAKE: u16 = 0;
pub const KEY_BREAK: u16 = 1;
pub const KEY_E0: u16 = 2;
pub const KEY_E1: u16 = 4;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MouseButtons {
    pub flags: u16, // 2-byte size to match original
}

impl MouseButtons {
    pub fn new() -> Self {
        MouseButtons { flags: 0 }
    }

    pub fn set_flag(&mut self, mask: u16, value: bool) {
        if value {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }

    pub fn check_flag(&self, mask: u16) -> bool {
        (self.flags & mask) != 0
    }
}

// Mouse button masks for easy use
pub const L_BUTTON_DOWN: u16 = 0x0001;
pub const L_BUTTON_UP: u16 = 0x0002;
pub const R_BUTTON_DOWN: u16 = 0x0004;
pub const R_BUTTON_UP: u16 = 0x0008;
pub const M_BUTTON_DOWN: u16 = 0x0010;
pub const M_BUTTON_UP: u16 = 0x0020;
pub const X_BUTTON1_DOWN: u16 = 0x0040;
pub const X_BUTTON1_UP: u16 = 0x0080;
pub const X_BUTTON2_DOWN: u16 = 0x0100;
pub const X_BUTTON2_UP: u16 = 0x0200;
pub const WHEEL: u16 = 0x0400;
pub const H_WHEEL: u16 = 0x0800;

impl RzControl {
    pub fn new(r#type: Type) -> Self {
        RzControl {
            unk1: 0,
            r#type,
            data: unsafe { std::mem::zeroed() },
        }
    }

    // Safe accessors for union fields
    pub unsafe fn mouse_data(&self) -> &MouseInputData {
        &self.data.mouse
    }

    pub unsafe fn keyboard_data(&self) -> &KeyboardInputData {
        &self.data.keyboard
    }

    pub unsafe fn mouse_data_mut(&mut self) -> &mut MouseInputData {
        &mut self.data.mouse
    }

    pub unsafe fn keyboard_data_mut(&mut self) -> &mut KeyboardInputData {
        &mut self.data.keyboard
    }
}

// Compile-time size checks
const _: () = assert!(std::mem::size_of::<MouseButtons>() == 2);
const _: () = assert!(std::mem::size_of::<RzControl>() == 32);
