use crate::{
    Sample,
    error::{Error, Result},
};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<const N: usize> {
    buffer: [Sample; N],
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N - 1
    }

    #[inline]
    pub fn available(&self) -> usize {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Acquire);

        if write >= read {
            write - read
        } else {
            N - read + write
        }
    }

    #[inline]
    pub fn free(&self) -> usize {
        self.capacity() - self.available()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.available() == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.free() == 0
    }

    #[inline]
    pub fn write(&mut self, data: &[Sample]) -> usize {
        let mut written = 0;

        for &sample in data {
            let write = self.write_pos.load(Ordering::Acquire);
            let read = self.read_pos.load(Ordering::Acquire);
            let next = (write + 1) % N;

            if next != read {
                self.buffer[write] = sample;
                self.write_pos.store(next, Ordering::Release);
                written += 1;
            } else {
                break; // Buffer full
            }
        }

        written
    }

    pub fn write_exact(&mut self, data: &[Sample]) -> Result<()> {
        if data.len() > self.free() {
            return Err(Error::BufferFull);
        }

        let written = self.write(data);
        debug_assert_eq!(written, data.len());
        Ok(())
    }

    #[inline]
    pub fn read(&mut self, data: &mut [Sample]) -> usize {
        let mut read_count = 0;

        for sample in data.iter_mut() {
            let read = self.read_pos.load(Ordering::Acquire);
            let write = self.write_pos.load(Ordering::Acquire);

            if read != write {
                *sample = self.buffer[read];
                self.read_pos.store((read + 1) % N, Ordering::Release);
                read_count += 1;
            } else {
                break; // Buffer empty
            }
        }

        read_count
    }

    pub fn read_exact(&mut self, data: &mut [Sample]) -> Result<()> {
        if data.len() > self.available() {
            return Err(Error::BufferEmpty);
        }

        let read = self.read(data);
        debug_assert_eq!(read, data.len());
        Ok(())
    }

    pub fn peek(&self, data: &mut [Sample]) -> usize {
        let mut read_pos = self.read_pos.load(Ordering::Acquire);
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let mut count = 0;

        for sample in data.iter_mut() {
            if read_pos != write_pos {
                *sample = self.buffer[read_pos];
                read_pos = (read_pos + 1) % N;
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    pub fn skip(&mut self, count: usize) -> usize {
        let available = self.available();
        let to_skip = count.min(available);

        let read = self.read_pos.load(Ordering::Acquire);
        let new_read = (read + to_skip) % N;
        self.read_pos.store(new_read, Ordering::Release);

        to_skip
    }

    pub fn clear(&mut self) {
        self.read_pos.store(0, Ordering::Release);
        self.write_pos.store(0, Ordering::Release);
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut rb: RingBuffer<16> = RingBuffer::new();

        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 15);

        let written = rb.write(&[1.0, 2.0, 3.0]);
        assert_eq!(written, 3);
        assert_eq!(rb.available(), 3);

        let mut output = [0.0; 3];
        let read = rb.read(&mut output);
        assert_eq!(read, 3);
        assert_eq!(output, [1.0, 2.0, 3.0]);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut rb: RingBuffer<8> = RingBuffer::new();

        rb.write(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let mut output = [0.0; 3];
        rb.read(&mut output);

        rb.write(&[6.0, 7.0, 8.0]);

        let mut output = [0.0; 5];
        let read = rb.read(&mut output);
        assert_eq!(read, 5);
        assert_eq!(output, [4.0, 5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn test_ring_buffer_full() {
        let mut rb: RingBuffer<4> = RingBuffer::new();

        let written = rb.write(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(written, 3);
        assert!(rb.is_full());
    }

    #[test]
    fn test_peek() {
        let mut rb: RingBuffer<16> = RingBuffer::new();
        rb.write(&[1.0, 2.0, 3.0]);

        let mut peek_buf = [0.0; 2];
        let peeked = rb.peek(&mut peek_buf);
        assert_eq!(peeked, 2);
        assert_eq!(peek_buf, [1.0, 2.0]);

        assert_eq!(rb.available(), 3);
    }
}
