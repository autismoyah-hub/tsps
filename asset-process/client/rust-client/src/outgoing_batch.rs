//! Original dw.af's FIFO whole-node staging into its 5000-byte buffer.
#[derive(Default, Debug)]
pub struct BatchQueue {
    frames: std::collections::VecDeque<Vec<u8>>,
    bytes: usize,
}
impl BatchQueue {
    pub const CAPACITY: usize = 5000;
    pub fn push(&mut self, frame: Vec<u8>) {
        self.bytes += frame.len();
        self.frames.push_back(frame);
    }
    pub fn pending_bytes(&self) -> usize {
        self.bytes
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    pub fn clear(&mut self) {
        self.frames.clear();
        self.bytes = 0;
    }
    /// Nodes leave the queue before the transport write, including a failed write.
    /// An oversized head produces an empty batch and leaves every node queued.
    pub fn take_batch(&mut self) -> Option<Vec<u8>> {
        if self.bytes == 0 {
            return None;
        }
        let mut batch = Vec::with_capacity(self.bytes.min(Self::CAPACITY));
        while self
            .frames
            .front()
            .is_some_and(|frame| frame.len() <= Self::CAPACITY - batch.len())
        {
            let frame = self.frames.pop_front().unwrap();
            self.bytes -= frame.len();
            batch.extend_from_slice(&frame);
        }
        Some(batch)
    }
}
