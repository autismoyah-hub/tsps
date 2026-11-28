//! Readiness for a bounded observation smoke, not a gameplay scheduling policy.
use std::time::Duration;
/// A title prompt cannot be completed by an unattended observation smoke.
/// Connecting/reconnecting has no logout reason and must keep waiting.
pub fn blocked(smoke: bool, logout_reason: Option<u8>) -> bool {
    smoke && logout_reason.is_some()
}
pub fn ready(
    elapsed: Duration,
    received_work_drained: bool,
    base: &[i32; 25],
    pending_widgets: usize,
    title: bool,
) -> bool {
    elapsed > Duration::from_secs(30)
        && received_work_drained
        && !title
        && pending_widgets == 0
        && base.iter().all(|&level| level > 0)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game_packet::{decode, Update},
        protocol::Packet,
        script_data::Data,
    };
    #[test]
    fn unattended_failure_is_distinct_from_connecting_and_interactive_retry() {
        assert!(!blocked(true, None));
        for reason in [0, 1, 2] {
            assert!(blocked(true, Some(reason)));
            assert!(!blocked(false, Some(reason)));
        }
        // The real observed rejection uses the ordinary Java response mapping.
        let response = crate::login_response::response(13, false);
        assert_eq!(response.index, 9);
        assert_eq!(response.lines.unwrap()[0], "Could not complete login.");
    }
    fn stat(data: &mut Data, index: u8, current: u8, xp: [u8; 4]) {
        let payload = [vec![current, current, index.wrapping_add(128)], xp.to_vec()].concat();
        let Update::Stat {
            index,
            boosted,
            invisible,
            experience,
        } = decode(&Packet {
            opcode: 46,
            payload,
        })
        .unwrap()
        .unwrap()
        else {
            panic!("stat packet")
        };
        data.stat(index, boosted, invisible, experience).unwrap();
    }
    #[test]
    fn initial_stat_packets_must_arrive_before_a_completed_smoke() {
        let mut data = Data::default();
        let elapsed = Duration::from_secs(60);
        assert!(!ready(elapsed, true, &data.base, 0, false));
        for index in 0..24 {
            stat(
                &mut data,
                index,
                if index == 3 { 10 } else { 1 },
                if index == 3 { [4, 130, 0, 0] } else { [0; 4] },
            );
        }
        assert_eq!((data.base[3], data.experience[3]), (10, 1154));
        assert!(!ready(elapsed, true, &data.base, 0, false));
        stat(&mut data, 24, 1, [0; 4]);
        assert!(ready(elapsed, true, &data.base, 0, false));
        // Zero current HP is a valid state; only unreceived base levels block.
        stat(&mut data, 3, 0, [4, 130, 0, 0]);
        assert!(ready(elapsed, true, &data.base, 0, false));
    }
    #[test]
    fn elapsed_time_does_not_override_pending_work_or_incomplete_rendering() {
        let base = [1; 25];
        let elapsed = Duration::from_secs(60);
        assert!(!ready(elapsed, false, &base, 0, false));
        assert!(!ready(elapsed, true, &base, 1, false));
        assert!(!ready(elapsed, true, &base, 0, true));
        assert!(!ready(Duration::from_secs(30), true, &base, 0, false));
        assert!(ready(elapsed, true, &base, 0, false));
    }
}
