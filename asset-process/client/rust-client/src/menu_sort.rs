//! Stable priority ordering from GameClient.in. Opcode1000 is an unswappable separator.
pub fn prioritize<T>(entries:&mut [T],opcode:impl Fn(&T)->i32) {
    for segment in entries.split_mut(|entry|opcode(entry)==1000) {
        segment.sort_by_key(|entry|opcode(entry)<1000);
    }
}
