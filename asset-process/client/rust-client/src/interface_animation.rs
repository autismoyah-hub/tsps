//! Interface model clocks follow ga.rt; cached sequences advance once per update.
use crate::{
    animation::Sequence, interface_state::Interfaces, widget_layout::Layout, Error, Result,
};
use std::collections::HashMap;
pub fn advance(frame: &mut usize, cycle: &mut i32, sequence: &Sequence, ticks: i32) -> Result<()> {
    if ticks < 0 {
        return Err(Error::Invalid("negative interface animation ticks"));
    }
    if sequence.cached_animation.is_some() {
        let count = sequence.cached_range[1] as i32 - sequence.cached_range[0] as i32;
        let mut next = (*frame as i32).wrapping_add(ticks);
        if next >= count {
            next = next.wrapping_sub(sequence.loop_frames);
            if next < 0 || next >= count {
                next = 0;
            }
        }
        *frame = next as usize;
    } else {
        if *frame >= sequence.frames.len() || sequence.frames.len() != sequence.lengths.len() {
            return Err(Error::Invalid("interface animation frame"));
        }
        *cycle = cycle.wrapping_add(ticks);
        let mut transitions = 0;
        while *cycle > sequence.lengths[*frame] as i32 {
            *cycle -= sequence.lengths[*frame] as i32;
            *frame += 1;
            if *frame >= sequence.frames.len() {
                let next = (*frame as i32).wrapping_sub(sequence.loop_frames);
                *frame = if next < 0 || next as usize >= sequence.frames.len() {
                    0
                } else {
                    next as usize
                };
            }
            transitions += 1;
            if transitions > 1_000_000 {
                return Err(Error::Invalid("nonprogressing interface animation"));
            }
        }
    }
    Ok(())
}
pub fn tick(
    state: &mut Interfaces,
    layout: &Layout,
    sequences: &HashMap<u32, Sequence>,
    ticks: i32,
) -> Result<()> {
    for &id in &layout.order {
        let p = &layout.widgets[&id];
        let w = state.widgets.get_mut(&id).unwrap();
        if p.hidden || w.kind != 6 {
            continue;
        }
        let r = state.runtime.get_mut(&id).unwrap();
        let sequence = w.sequences[usize::from(r.condition)];
        if sequence >= 0 {
            let seq = sequences
                .get(&(sequence as u32))
                .ok_or(Error::Invalid("interface sequence missing"))?;
            advance(&mut r.frame, &mut r.frame_cycle, seq, ticks)?;
        }
        if !w.if3 {
            for (axis, speed) in [0, 2].into_iter().zip(r.rotation_speed) {
                w.model_angles[axis] = ((w.model_angles[axis] as i32)
                    .wrapping_add((speed as i32).wrapping_mul(ticks))
                    & 2047) as u16;
            }
        }
    }
    Ok(())
}
