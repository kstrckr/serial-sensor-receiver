const SIZE: usize = 10000;

pub struct CircleBuffer<T> {
    read_index: IndexCounter,
    write_index: IndexCounter,
    buffered_values: u32,
    pub buffer: [T; SIZE],
}

impl CircleBuffer<u8> {
    pub fn new() -> Self {
        let cb: CircleBuffer<u8> = CircleBuffer {
            read_index: IndexCounter {
                index: 0,
                max_size: SIZE,
            },
            write_index: IndexCounter {
                index: 0,
                max_size: SIZE,
            },
            buffer: [0; SIZE],
            buffered_values: 0,
        };
        cb
    }

    pub fn percent_utilized(&self) -> f32 {
        (self.buffered_values as f32 /SIZE as f32) * 100.0
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut iter = data.into_iter();
        while let Some(n) = iter.next() {
            self.buffer[self.write_index.index] = *n;
            self.write_index.increment();
            self.buffered_values += 1;
        }
        self.write_index.index
    }

    fn read(&mut self) -> u8 {
        let val = self.buffer[self.read_index.index];
        self.buffer[self.read_index.index] = 0;
        self.read_index.increment();
        self.buffered_values -= 1;
        val
    }

    pub fn read_cobs_frame(&mut self) -> Option<Vec<u8>> {
        if self.buffered_values < 18 {
            None
        } else {
            let mut frame: Vec<u8> = Vec::new();
            let mut reading = true;
            while reading {
                let byte = self.read();
                frame.push(byte);
                if byte == 0 {
                    reading = false;
                }
            }
            if frame.len() == 18 {
                Some(frame)
            } else {
                None
            }
        }
    }
}

struct IndexCounter {
    index: usize,
    max_size: usize,
}

impl IndexCounter {
    fn increment(&mut self) -> usize {
        let next_val = self.index + 1;
        if next_val >= self.max_size {
            self.index = 0;
        } else {
            self.index = next_val;
        }
        next_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_counter_increment() {
        let mut ic = IndexCounter {
            index: 0,
            max_size: 3,
        };

        ic.increment();
        ic.increment();
        ic.increment();


        assert_eq!(0, ic.index)
    }

    #[test]
    fn circle_buff_constructor() {
        let mut cb: CircleBuffer<u8> = CircleBuffer::new();
        let i = cb.write(&[0]);
        assert_eq!(i, 1)
    }

    #[test]
    fn circle_buff_write() {
        let mut cb: CircleBuffer<u8> = CircleBuffer::new();
        let i = cb.write(&[0, 0, 0]);
        assert_eq!(cb.write_index.index, 3)
    }

    #[test]
    fn circle_buff_read() {
        let mut cb: CircleBuffer<u8> = CircleBuffer::new();
        let i = cb.write(&[0, 1, 2, 3]);
        assert_eq!(cb.read_index.index, 0);
        let first_read = cb.read();
        assert_eq!(first_read, 0);
        assert_eq!(cb.read_index.index, 1);
        let second_read = cb.read();
        assert_eq!(second_read, 1);
        assert_eq!(cb.read_index.index, 2);
    }

    #[test]
    fn circle_buff_read_cobs() {
        let mut cb: CircleBuffer<u8> = CircleBuffer::new();
        let i = cb.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]);
        let frame = cb.read_cobs_frame().unwrap();
        assert_eq!(frame, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0])
    }
}
