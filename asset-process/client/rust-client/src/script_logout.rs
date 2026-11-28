//! LOGOUT client script opcode from bj.gc/jb.br.
//!
//! The opcode only arms GameClient.jp (stored as -578161982, read with
//! multiplier -977465187); it sends no packet. The server answers the widget
//! button with LOGOUT, and kk.bo treats a lost connection as a logout rather
//! than a reconnect while the timer is still positive.
pub const OPCODE: u16 = 5630;
/// GameClient.jp value written by opcode 5630 and IF1 content type 205 (ot.gu).
pub const TIMER: i32 = 250;
pub fn execute(op: u16, logout_timer: &mut i32) -> bool {
    if op != OPCODE {
        return false;
    }
    *logout_timer = TIMER;
    true
}
