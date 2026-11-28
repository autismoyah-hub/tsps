//! Owned asynchronous Windows PCM output buffers.
/// Extends the WinMM 32-bit position counter before converting its returned units.
#[derive(Default)]
pub struct Position {
    format: u32,
    raw: u32,
    units: u64,
}
impl Position {
    pub fn observe(&mut self, format: u32, raw: u32, channels: u16) -> crate::Result<u64> {
        if channels == 0 || !matches!(format, 1 | 2 | 4) {
            return Err(crate::Error::Invalid("audio playback position format"));
        }
        if self.format != 0 && self.format != format {
            return Err(crate::Error::Invalid("audio playback position format changed"));
        }
        self.units += if self.format == 0 { raw as u64 } else { raw.wrapping_sub(self.raw) as u64 };
        self.format = format;
        self.raw = raw;
        Ok(match format {
            1 => self.units * 22050 / 1000,
            2 => self.units,
            _ => self.units / (channels as u64 * 2),
        })
    }
}
#[cfg(all(windows, not(feature = "offscreen")))]
mod windows {
    use crate::{Error, Result};
    use std::{collections::VecDeque, ptr};
    #[repr(C)]
    struct Format {
        tag: u16,
        channels: u16,
        rate: u32,
        bytes_per_second: u32,
        alignment: u16,
        bits: u16,
        extra: u16,
    }
    #[repr(C)]
    struct Header {
        data: *mut i8,
        length: u32,
        recorded: u32,
        user: usize,
        flags: u32,
        loops: u32,
        next: *mut Header,
        reserved: usize,
    }
    #[repr(C)]
    struct MmTime { kind: u32, value: [u32; 2] }
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn waveOutOpen(
            handle: *mut usize,
            device: u32,
            format: *const Format,
            callback: usize,
            instance: usize,
            flags: u32,
        ) -> u32;
        fn waveOutPrepareHeader(handle: usize, header: *mut Header, size: u32) -> u32;
        fn waveOutUnprepareHeader(handle: usize, header: *mut Header, size: u32) -> u32;
        fn waveOutWrite(handle: usize, header: *mut Header, size: u32) -> u32;
        fn waveOutGetPosition(handle: usize, time: *mut MmTime, size: u32) -> u32;
        fn waveOutReset(handle: usize) -> u32;
        fn waveOutClose(handle: usize) -> u32;
        fn waveOutGetErrorTextW(code: u32, text: *mut u16, length: u32) -> u32;
    }
    fn check(code: u32) -> Result<()> {
        if code == 0 {
            return Ok(());
        }
        let mut message = [0u16; 256];
        unsafe {
            waveOutGetErrorTextW(code, message.as_mut_ptr(), 256);
        }
        let len = message
            .iter()
            .position(|&v| v == 0)
            .unwrap_or(message.len());
        Err(Error::Io(format!(
            "Windows audio {code}: {}",
            String::from_utf16_lossy(&message[..len])
        )))
    }
    struct Buffer {
        header: Box<Header>,
        _samples: Vec<i16>,
    }
    pub struct Output {
        handle: usize,
        buffers: VecDeque<Buffer>,
        channels: u16,
        submitted: u64,
        position: super::Position,
    }
    impl Output {
        pub fn open() -> Result<Self> {
            Self::open_channels(1)
        }
        pub fn open_stereo() -> Result<Self> {
            Self::open_channels(2)
        }
        fn open_channels(channels: u16) -> Result<Self> {
            let format = Format {
                tag: 1,
                channels,
                rate: 22050,
                bytes_per_second: 44100 * channels as u32,
                alignment: 2 * channels,
                bits: 16,
                extra: 0,
            };
            let mut handle = 0;
            check(unsafe { waveOutOpen(&mut handle, u32::MAX, &format, 0, 0, 0) })?;
            Ok(Self {
                handle,
                buffers: VecDeque::new(),
                channels,
                submitted: 0,
                position: super::Position::default(),
            })
        }
        fn reap(&mut self) -> Result<()> {
            while self
                .buffers
                .front()
                .is_some_and(|b| unsafe { ptr::read_volatile(&b.header.flags) } & 1 != 0)
            {
                let b = self.buffers.front_mut().unwrap();
                check(unsafe {
                    waveOutUnprepareHeader(
                        self.handle,
                        &mut *b.header,
                        std::mem::size_of::<Header>() as u32,
                    )
                })?;
                self.buffers.pop_front();
            }
            Ok(())
        }
        pub fn queued_frames(&mut self) -> Result<i32> {
            self.reap()?;
            let mut time = MmTime { kind: 2, value: [0; 2] };
            check(unsafe { waveOutGetPosition(self.handle, &mut time, std::mem::size_of::<MmTime>() as u32) })?;
            let played = self.position.observe(time.kind, time.value[0], self.channels)?;
            i32::try_from(self.submitted.saturating_sub(played)).map_err(|_| Error::Invalid("audio queue position"))
        }
        pub fn write(&mut self, mut samples: Vec<i16>) -> Result<()> {
            self.reap()?;
            if samples.len() % self.channels as usize != 0 {
                return Err(Error::Invalid("partial audio sample frame"));
            }
            if samples.is_empty() {
                return Ok(());
            }
            if self.buffers.len() >= 100 {
                return Err(Error::Invalid("audio output queue exceeds two seconds"));
            }
            let length = samples
                .len()
                .checked_mul(2)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or(Error::Invalid("audio output length"))?;
            let mut header = Box::new(Header {
                data: samples.as_mut_ptr().cast(),
                length,
                recorded: 0,
                user: 0,
                flags: 0,
                loops: 0,
                next: ptr::null_mut(),
                reserved: 0,
            });
            let size = std::mem::size_of::<Header>() as u32;
            check(unsafe { waveOutPrepareHeader(self.handle, &mut *header, size) })?;
            let code = unsafe { waveOutWrite(self.handle, &mut *header, size) };
            if code != 0 {
                unsafe {
                    waveOutUnprepareHeader(self.handle, &mut *header, size);
                }
                return check(code);
            }
            self.submitted += (samples.len() / self.channels as usize) as u64;
            self.buffers.push_back(Buffer {
                header,
                _samples: samples,
            });
            Ok(())
        }
    }
    impl Drop for Output {
        fn drop(&mut self) {
            unsafe {
                waveOutReset(self.handle);
                for buffer in &mut self.buffers {
                    waveOutUnprepareHeader(
                        self.handle,
                        &mut *buffer.header,
                        std::mem::size_of::<Header>() as u32,
                    );
                }
                waveOutClose(self.handle);
            }
        }
    }
}
#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::Output;
#[cfg(feature = "offscreen")]
mod null {
    //! #233 offscreen PCM device: no OS device and every sample is discarded. The queue drains
    //! at the nominal 22050 Hz against a monotonic clock and only ever holds frames actually
    //! written: elapsed time with nothing queued is not credited, so after an underrun the
    //! next writes are queued, not instantly drained. This keeps the shared mixer advancing at
    //! about the rate a real device would pull it. No device, timing, PCM or fidelity claim.
    use crate::{Error, Result};
    use std::time::Duration;
    const RATE: u128 = 22050;
    const MICROS: u128 = 1_000_000;
    /// The pure queue state, advanced to an explicit monotonic `now`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct NullQueue {
        capacity: u64,
        queued: u64,
        last: Duration,
        /// Sub-frame drain progress, in frame-microseconds (always below one frame).
        remainder: u64,
        submitted: u64,
    }
    impl NullQueue {
        /// A fresh queue: empty, timestamped `now`, no remainder (every open and reopen).
        pub fn open_at(capacity: i32, now: Duration) -> Result<Self> {
            if capacity <= 0 {
                return Err(Error::Invalid("null audio capacity"));
            }
            Ok(Self { capacity: capacity as u64, queued: 0, last: now, remainder: 0, submitted: 0 })
        }
        /// Drains the frames due since the last advance. A clock that went backward is an
        /// error and changes nothing. Draining saturates at the queued frames; elapsed time
        /// beyond them (and any sub-frame remainder) is discarded, never carried as credit.
        pub fn advance_to(&mut self, now: Duration) -> Result<()> {
            let elapsed = now.checked_sub(self.last).ok_or(Error::Invalid("null audio clock went backward"))?;
            // At most (u64::MAX + 1) * 10^6 * 22050 + 10^6 < 2^128.
            let total = elapsed.as_micros() * RATE + self.remainder as u128;
            let frames = total / MICROS;
            if frames >= self.queued as u128 {
                self.queued = 0;
                self.remainder = 0;
            } else {
                self.queued -= frames as u64;
                self.remainder = (total % MICROS) as u64;
            }
            self.last = now;
            Ok(())
        }
        pub fn queued_at(&mut self, now: Duration) -> Result<i32> {
            self.advance_to(now)?;
            i32::try_from(self.queued).map_err(|_| Error::Invalid("null audio queue"))
        }
        /// Queues `samples / 2` stereo frames. A partial frame, or more than the capacity
        /// left, is an error and adds nothing.
        pub fn write_at(&mut self, samples: usize, now: Duration) -> Result<()> {
            self.advance_to(now)?;
            if samples % 2 != 0 {
                return Err(Error::Invalid("partial audio sample frame"));
            }
            let frames = (samples / 2) as u64;
            self.queued = self.queued.checked_add(frames).filter(|&queued| queued <= self.capacity)
                .ok_or(Error::Invalid("null audio queue over capacity"))?;
            self.submitted = self.submitted.saturating_add(frames);
            Ok(())
        }
        pub fn capacity(&self) -> u64 { self.capacity }
        pub fn queued(&self) -> u64 { self.queued }
        pub fn remainder(&self) -> u64 { self.remainder }
        pub fn submitted(&self) -> u64 { self.submitted }
    }
    /// The PCM worker's `Output` under `offscreen`: a `NullQueue` on the monotonic clock.
    pub struct NullOutput {
        queue: NullQueue,
    }
    impl NullOutput {
        pub fn open_null(capacity: i32) -> Result<Self> {
            Ok(Self { queue: NullQueue::open_at(capacity, now())? })
        }
        pub fn queued_frames(&mut self) -> Result<i32> {
            self.queue.queued_at(now())
        }
        pub fn write(&mut self, samples: Vec<i16>) -> Result<()> {
            self.queue.write_at(samples.len(), now())
        }
        pub fn queue(&self) -> &NullQueue {
            &self.queue
        }
    }
    /// Monotonic time since this process first used a null device. Tests inject the clock
    /// per thread instead.
    fn now() -> Duration {
        #[cfg(test)]
        {
            if let Some(now) = TEST_NOW.with(|now| now.get()) {
                return now;
            }
        }
        static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        EPOCH.get_or_init(std::time::Instant::now).elapsed()
    }
    #[cfg(test)]
    thread_local! {
        static TEST_NOW: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
    }
    /// Test-only: the clock this thread's null devices read.
    #[cfg(test)]
    pub(crate) fn set_test_now(now: Duration) {
        TEST_NOW.with(|cell| cell.set(Some(now)));
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn ms(n: u64) -> Duration { Duration::from_millis(n) }
        fn us(n: u64) -> Duration { Duration::from_micros(n) }

        #[test]
        fn null_audio_drains_at_nominal_rate_with_exact_remainder() {
            let mut queue = NullQueue::open_at(32768, ms(0)).unwrap();
            queue.write_at(2 * 30000, ms(0)).unwrap();
            // 1 ms = 22.05 frames: 22 drained, 0.05 frame (50,000 frame-us) carried.
            queue.advance_to(ms(1)).unwrap();
            assert_eq!((queue.queued(), queue.remainder()), (29978, 50_000));
            // Twenty more 1 ms steps: the remainder carries, 21 * 22.05 = 463.05.
            for n in 2..=21 { queue.advance_to(ms(n)).unwrap(); }
            assert_eq!((queue.queued(), queue.remainder()), (30000 - 463, 50_000));
            // One second from the start drains exactly 22,050 frames in total.
            queue.advance_to(ms(1000)).unwrap();
            assert_eq!((queue.queued(), queue.remainder()), (30000 - 22050, 0));
            // Sub-microsecond time is ignored, not rounded up.
            queue.advance_to(ms(1000) + Duration::from_nanos(999)).unwrap();
            assert_eq!(queue.queued(), 30000 - 22050);
            assert_eq!(queue.submitted(), 30000);
        }

        #[test]
        fn null_audio_long_idle_then_write_has_no_underrun_credit() {
            let mut queue = NullQueue::open_at(4096, ms(0)).unwrap();
            queue.write_at(2 * 1024, ms(0)).unwrap();
            queue.advance_to(ms(3_600_000)).unwrap();
            assert_eq!((queue.queued(), queue.remainder()), (0, 0), "an hour drains the queue and keeps no credit");
            queue.write_at(2 * 1024, ms(3_600_000)).unwrap();
            assert_eq!(queue.queued_at(ms(3_600_000)).unwrap(), 1024, "new frames are queued, not pre-drained");
            assert_eq!(queue.queued_at(ms(3_600_001)).unwrap(), 1024 - 22);
            // An underrun mid-step also discards the unused part of that step.
            let mut short = NullQueue::open_at(4096, ms(0)).unwrap();
            short.write_at(2 * 10, ms(0)).unwrap();
            short.advance_to(ms(1)).unwrap();
            assert_eq!((short.queued(), short.remainder()), (0, 0));
            short.write_at(2 * 100, ms(1)).unwrap();
            assert_eq!(short.queued_at(us(1_500)).unwrap(), 100 - 11, "0.5 ms = 11.025 frames from the write");
        }

        #[test]
        fn null_audio_reopen_resets_queue_clock_and_remainder() {
            let mut queue = NullQueue::open_at(4096, ms(0)).unwrap();
            queue.write_at(2 * 2048, ms(0)).unwrap();
            queue.advance_to(ms(1)).unwrap();
            assert_ne!(queue.remainder(), 0);
            // Pump's capacity growth closes and reopens with capacity + 1024.
            let reopened = NullQueue::open_at(4096 + 1024, ms(5)).unwrap();
            assert_eq!((reopened.capacity(), reopened.queued(), reopened.remainder(), reopened.submitted()), (5120, 0, 0, 0));
            let mut reopened = reopened;
            assert_eq!(reopened.queued_at(ms(5)).unwrap(), 0);
            assert!(reopened.advance_to(ms(4)).is_err(), "the reopen time is the new clock origin");
            for capacity in [0, -1, i32::MIN] {
                assert!(NullQueue::open_at(capacity, ms(0)).is_err(), "{capacity}");
            }
        }

        #[test]
        fn null_audio_backward_clock_is_an_error_and_leaves_state_unchanged() {
            let mut queue = NullQueue::open_at(4096, ms(10)).unwrap();
            queue.write_at(2 * 1000, ms(10)).unwrap();
            queue.advance_to(us(10_500)).unwrap();
            let before = queue.clone();
            assert!(queue.advance_to(ms(9)).is_err());
            assert!(queue.queued_at(us(10_499)).is_err());
            assert!(queue.write_at(2, ms(10)).is_err());
            assert_eq!(queue, before, "a refused clock changes nothing");
            queue.advance_to(ms(11)).unwrap();
            assert_eq!(queue.queued(), 1000 - 22, "a later valid clock drains from the last good time");
        }

        #[test]
        fn null_audio_arithmetic_does_not_overflow() {
            let mut queue = NullQueue::open_at(i32::MAX, Duration::ZERO).unwrap();
            queue.write_at(2 * (i32::MAX as usize), Duration::ZERO).unwrap();
            assert_eq!(queue.queued_at(Duration::ZERO).unwrap(), i32::MAX);
            // The largest elapsed time a Duration holds drains everything without overflow.
            queue.advance_to(Duration::MAX).unwrap();
            assert_eq!((queue.queued(), queue.remainder()), (0, 0));
            assert!(queue.advance_to(Duration::MAX).is_ok(), "zero elapsed at the maximum clock");
            // Frame counts far beyond the capacity are refused by checked arithmetic.
            for samples in [usize::MAX - 1, 2 * (i32::MAX as usize) + 2] {
                assert!(queue.write_at(samples, Duration::MAX).is_err(), "{samples}");
                assert_eq!(queue.queued(), 0);
            }
            let mut full = NullQueue::open_at(i32::MAX, Duration::ZERO).unwrap();
            full.write_at(2 * (i32::MAX as usize), Duration::ZERO).unwrap();
            assert!(full.write_at(2, Duration::ZERO).is_err());
            assert_eq!(full.submitted(), i32::MAX as u64);
        }

        #[test]
        fn null_audio_partial_stereo_write_is_refused() {
            let mut queue = NullQueue::open_at(4096, ms(0)).unwrap();
            queue.write_at(2 * 100, ms(0)).unwrap();
            for samples in [1, 3, 1023] {
                assert!(queue.write_at(samples, ms(0)).is_err(), "{samples}");
            }
            assert_eq!((queue.queued(), queue.submitted()), (100, 100));
            queue.write_at(0, ms(0)).unwrap();
            assert_eq!(queue.queued(), 100);
        }

        #[test]
        fn null_audio_repeated_same_time_writes_stop_at_capacity() {
            let mut queue = NullQueue::open_at(4096, ms(7)).unwrap();
            for block in 1..=8 {
                queue.write_at(2 * 512, ms(7)).unwrap();
                assert_eq!(queue.queued(), block * 512);
            }
            assert!(queue.write_at(2 * 512, ms(7)).is_err(), "a ninth block exceeds 4096");
            assert!(queue.write_at(2, ms(7)).is_err(), "even one frame over is refused");
            assert_eq!((queue.queued(), queue.submitted()), (4096, 4096));
            // Draining makes room again: 1 ms frees 22 frames.
            queue.write_at(2 * 22, ms(8)).unwrap();
            assert_eq!(queue.queued(), 4096);
        }

        #[test]
        fn null_output_reads_the_injected_clock() {
            set_test_now(ms(100));
            let mut output = NullOutput::open_null(4096).unwrap();
            output.write(vec![0; 2 * 512]).unwrap();
            assert!(output.write(vec![0; 3]).is_err());
            set_test_now(ms(110));
            assert_eq!(output.queued_frames().unwrap(), 512 - 220);
            set_test_now(ms(109));
            assert!(output.queued_frames().is_err());
            assert_eq!(output.queue().queued(), 512 - 220);
        }
    }
}
#[cfg(feature = "offscreen")]
pub use null::{NullOutput, NullOutput as Output, NullQueue};
#[cfg(all(test, feature = "offscreen"))]
pub(crate) use null::set_test_now;
