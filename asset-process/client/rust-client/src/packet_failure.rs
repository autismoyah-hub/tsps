//! Recovery choice at the original GameClient.ib exception boundary.
use crate::Error;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Reconnect,
    Logout,
}
pub fn action(error: &Error, logout_timer: i32) -> Action {
    // Out-of-range incoming opcodes explicitly throw IOException in GameClient.ib.
    if logout_timer <= 0 && matches!(error, Error::Io(_) | Error::Invalid("unknown opcode")) {
        Action::Reconnect
    } else {
        Action::Logout
    }
}
