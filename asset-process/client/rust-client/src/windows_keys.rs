//! Windows virtual keys mapped to revision 239 client key codes.
pub const KEYS: [i32; 256] = [
    -1, -1, -1, -1, -1, -1, -1, -1, 85, 80, 84, -1, 91, 84, -1, -1, 81, 82, 86, -1, -1, -1, -1, -1,
    -1, -1, -1, 13, -1, -1, -1, -1, 83, 104, 105, 103, 102, 96, 98, 97, 99, -1, -1, -1, -1, 100,
    101, -1, 25, 16, 17, 18, 19, 20, 21, 22, 23, 24, -1, -1, -1, -1, -1, -1, -1, 48, 68, 66, 50,
    34, 51, 52, 53, 39, 54, 55, 56, 70, 69, 40, 41, 32, 35, 49, 36, 38, 67, 33, 65, 37, 64, -1, -1,
    -1, -1, -1, 228, 231, 227, 233, 224, 219, 225, 230, 226, 232, 89, 87, -1, 88, 229, 90, 1, 2, 3,
    4, 5, 6, 7, 8, 9, 10, 11, 12, -1, -1, -1, 101, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 100, -1, 87, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 57,
    27, 71, 26, 72, 73, 58, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 42, 74, 43, 59, 28, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];

/// AWT WmChar converts Win32 carriage return to Java KEY_TYPED line feed.
pub fn character(value: u16) -> u16 {
    if value == 13 {
        10
    } else {
        value
    }
}

/// AWT synthesizes KEY_TYPED for Delete because Windows emits no WM_CHAR.
pub fn down_character(virtual_key: usize) -> Option<u16> {
    (virtual_key == 46).then_some(127)
}

/// AWT WmChar accepts ordinary and system characters, except system Alt+Space.
pub fn character_message(message: u32, value: u16, flags: isize) -> Option<u16> {
    match message {
        0x102 => Some(character(value)),
        0x106 if !(value == 32 && flags & (1 << 29) != 0) => Some(character(value)),
        _ => None,
    }
}
