use std::time::Duration;
use std::io::prelude::*;
use std::io::{self};

use circle_buff::CircleBuffer;

fn follow_stuffed_bytes(next_value: usize, frame: &[u8]) -> bool {
    if next_value > frame.len() || next_value == 0 || frame[frame.len() - 1] != 0 {
        false
    } else {
        match frame[next_value] {
            0 => true,
            _ => {
                let next_index = next_value + frame[next_value] as usize;
                follow_stuffed_bytes(next_index, &frame)
            }
        }
    }
}

fn cobs_frame_is_valid(frame: &[u8]) -> bool {
    follow_stuffed_bytes(frame[0] as usize, &frame)
}

#[test]
fn valid_cobs_frame() {
    let valid_frame = [1, 1, 3, 10, 10, 1, 3, 11, 11, 1, 3, 12, 12, 1, 1, 0];
    assert_eq!(cobs_frame_is_valid(&valid_frame), true);
}

#[test]
fn invalid_cobs_frame_1() {
    let invalid_frame = [173, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 1, 1, 0];
    assert_eq!(cobs_frame_is_valid(&invalid_frame), false);
}

#[test]
fn invalid_cobs_frame_2() {
    let invalid_frame = [0, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 1, 1, 0];
    assert_eq!(cobs_frame_is_valid(&invalid_frame), false);
}

#[test]
fn invalid_cobs_frame_3() {
    let invalid_frame = [173, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 1, 1, 99];
    assert_eq!(cobs_frame_is_valid(&invalid_frame), false);
}

fn deserialize_cobs_frame (frame: &[u8]) -> Result<[u8; 16], &[u8]> {
    if cobs_frame_is_valid(frame) {
        if frame[0] == frame.len() as u8 + 1 {
            Ok([frame[1], frame[2], frame[3], frame[4], frame[5], frame[6], frame[7], frame[8], frame[9], frame[11], frame[12], frame[13], frame[14], frame[15], frame[16], frame[17]])
        } else {
            let mut indeces_to_zero = Vec::new();
            let mut parsed_frame = [0; 16];
            let mut next_index = frame[0] as usize;
            loop {
                if frame[next_index] == 0 {
                    break;
                }
                indeces_to_zero.push(next_index);
                next_index += frame[next_index] as usize;

            }
            for i in 1..frame.len() - 1 {
                parsed_frame[i - 1] = frame[i];
            }
            for i in &indeces_to_zero {
                let i: usize = *i as usize - 1;
                parsed_frame[i] = 0;
            }
            Ok(parsed_frame)
        }
    } else {
        Err(frame)
    }
}


#[test]
fn convert_valid_frame() {
    let valid_frame = [1, 1, 3, 10, 10, 1, 3, 11, 11, 1, 5, 12, 12, 1, 1, 0];
    let data = deserialize_cobs_frame(&valid_frame).expect("failed unwrapping");
    println!("{:?}", data);
    assert_eq!(data, [0, 0, 10, 10, 0, 0, 11, 11, 0, 0, 12, 12, 1, 1]);
}

fn format_data_to_text(d: [u8; 16]) -> (f32, f32, f32, u32) {
    let x = [d[0], d[1], d[2], d[3]];
    let y = [d[4], d[5], d[6], d[7]];
    let z = [d[8], d[9], d[10], d[11]];
    let counter = [d[12], d[13], d[14], d[15]];

    let x_f32: f32 = f32::from_le_bytes(x);
    let y_f32: f32 = f32::from_le_bytes(y);
    let z_f32: f32 = f32::from_le_bytes(z);
    let counter_u32 = u32::from_le_bytes(counter);
    (x_f32, y_f32, z_f32, counter_u32)
}

fn main() {
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    let port_name = "COM5";
    let baud_rate = 115_200;

    let port = serialport::new(port_name, baud_rate)
    .timeout(Duration::from_millis(10))
    .open();


    let mut count = 0;
    match port {
        Ok(mut port) => {
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);

            let mut cb: CircleBuffer<u8> = CircleBuffer::new();
            println!("\x1b[2J");
            let mut serial_buf: [u8; 18] = [0; 18];

            let mut x_sign = '-';
            let mut y_sign = '-';
            let mut z_sign = '-';

            loop {
                // let mut serial_buf: Vec<u8> = vec![0; 18];

                match port.read(&mut serial_buf) {
                    Ok(t) => {
                        cb.write(&serial_buf[0..t]);
                        let p = cb.percent_utilized();
                        let frame = cb.read_cobs_frame();

                        match frame {
                            Some(x) => {
                                let data = deserialize_cobs_frame(&x);
                                match data {
                                    Ok(d) => {
                                        count += 1;
                                        let (mut x, mut y, mut z, counter) = format_data_to_text(d);
                                        if x >= 0.0 {
                                            x_sign = '+';
                                        } else {
                                            x_sign = '-';
                                            x = x.abs();
                                        }
                                        if y >= 0.0 {
                                            y_sign = '+';
                                        } else {
                                            y_sign = '-';
                                            y = y.abs();
                                        }
                                        if z >= 0.0 {
                                            z_sign = '+';
                                        } else {
                                            z_sign = '-';
                                            z = z.abs();
                                        }
                                        println!("\x1b[2J\x1b[Hbuffer_utilization: {}%\nlocal_count/remote_count: {}/{}\nx: {}{}\ny: {}{}\nz: {}{}", p, count, counter, x_sign, x as i32, y_sign, y as i32, z_sign, z as i32);
                                    }
                                    Err(e) => {
                                        println!("{:?} - error", e);
                                        // port.clear(serialport::ClearBuffer::Input).expect("Failed to flush");
                                    }
                                }
                            }
                            None => {
                                // println!("waiting...")
                            }
                        }


                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
                // println!("{}/{}", count, error);
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}