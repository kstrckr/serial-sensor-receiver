
pub struct CircleBuffer<T> {
    read_index: IndexCounter,
    write_index: IndexCounter,
    pub buffer: [T; 1000],
}

impl CircleBuffer<u32> {
    pub fn new(size: usize) -> Self {
        println!("{}", size);
        let cb: CircleBuffer<u32> = CircleBuffer {
            read_index: IndexCounter {
                index: 0,
                max_size: size,
            },
            write_index: IndexCounter {
                index: 0,
                max_size: size,
            },
            buffer: [0; 1000],
        };
        cb
    }

    fn write(&mut self, data: &[u32]) -> usize {
        let mut iter = data.into_iter();
        println!("{:?}", iter.len());
        while let Some(n) = iter.next() {
            println!("{}", self.write_index.index);
            self.buffer[self.write_index.index] = *n;
            self.write_index.increment();
        }
        self.write_index.index
    }

    fn read(&mut self) -> u32 {
        let val = self.buffer[self.read_index.index];
        self.read_index.increment();
        val
    }

    fn read_cobs_frame(&mut self) -> Vec<u32> {
        let mut frame: Vec<u32> = Vec::new();
        let mut reading = true;
        while reading {
            let byte = self.read();
            frame.push(byte);
            if byte == 0 {
                reading = false;
            }
        }
        frame
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
        let mut cb: CircleBuffer<u32> = CircleBuffer::new(10);
        let i = cb.write(&[0]);
        assert_eq!(i, 1)
    }

    #[test]
    fn circle_buff_write() {
        let mut cb: CircleBuffer<u32> = CircleBuffer::new(10);
        let i = cb.write(&[0, 0, 0]);
        assert_eq!(cb.write_index.index, 3)
    }

    #[test]
    fn circle_buff_read() {
        let mut cb: CircleBuffer<u32> = CircleBuffer::new(10);
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
        let mut cb: CircleBuffer<u32> = CircleBuffer::new(10);
        let i = cb.write(&[1,2,3,4,5,0,1]);
        let frame = cb.read_cobs_frame();
        assert_eq!(frame, [1, 2, 3, 4, 5, 0])
    }
}
