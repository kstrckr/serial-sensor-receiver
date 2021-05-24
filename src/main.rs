use std::time::Duration;
use std::io::{self, Write};

fn follow_stuffed_bytes(next_value: usize, frame: &[u8; 14]) -> bool {
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

fn cobs_frame_is_valid(frame: [u8; 14]) -> bool {
    follow_stuffed_bytes(frame[0] as usize, &frame)
}


#[test]
fn valid_cobs_frame() {
    let valid_frame = [1, 1, 3, 10, 10, 1, 3, 11, 11, 1, 3, 12, 12, 0];
    assert_eq!(cobs_frame_is_valid(valid_frame), true);
}

#[test]
fn invalid_cobs_frame_1() {
    let invalid_frame = [173, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 0];
    assert_eq!(cobs_frame_is_valid(invalid_frame), false);
}

#[test]
fn invalid_cobs_frame_2() {
    let invalid_frame = [0, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 0];
    assert_eq!(cobs_frame_is_valid(invalid_frame), false);
}

#[test]
fn invalid_cobs_frame_3() {
    let invalid_frame = [173, 66, 4, 128, 81, 195, 0, 185, 66, 4, 128, 69, 195, 99];
    assert_eq!(cobs_frame_is_valid(invalid_frame), false);
}

fn deserialize_cobs_frame (frame: [u8; 14]) -> Result<[u8; 12], & 'static str> {
    if cobs_frame_is_valid(frame) {
        if frame[0] == frame.len() as u8 + 1 {
            Ok([frame[1], frame[2], frame[3], frame[4], frame[5], frame[6], frame[7], frame[8], frame[9], frame[11], frame[12], frame[13]])
        } else {
            let mut indeces_to_zero = Vec::new();
            let mut parsed_frame = [0; 12];
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
        Err("Invalid frame")
    }
}

#[test]
fn convert_valid_frame() {
    let valid_frame = [1, 1, 3, 10, 10, 1, 3, 11, 11, 1, 3, 12, 12, 0];
    let data = deserialize_cobs_frame(valid_frame).expect("failed unwrapping");
    println!("{:?}", data);
    assert_eq!(data, [0, 0, 10, 10, 0, 0, 11, 11, 0, 0, 12, 12]);
}


fn main() {
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    let port_name = "COM4";
    let baud_rate = 115_200;

    let port = serialport::new(port_name, baud_rate)
    .timeout(Duration::from_millis(10))
    .open();

    match port {
        Ok(mut port) => {
            // let mut serial_buf: Vec<u8> = vec![0; 4];
            let mut serial_buf: [u8; 14] = [0; 14];
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            let mut cycle = 0;
            loop {
                match port.read(& mut serial_buf) {
                    Ok(t) => {
                        if cobs_frame_is_valid(serial_buf) && t == 14 {
                            let data: [u8; 12] = deserialize_cobs_frame(serial_buf).expect("Fail");
                            let x = [data[0], data[1], data[2], data[3]];
                            let y = [data[4], data[5], data[6], data[7]];
                            let z = [data[8], data[9], data[10], data[11]];

                            let x_f32: f32 = f32::from_le_bytes(x);
                            let y_f32: f32 = f32::from_le_bytes(y);
                            let z_f32: f32 = f32::from_le_bytes(z);
                            // println!("bytes: {:?}, {:?}, {:?}", x, y, z);
                            println!("{}:{} value: {}, {}, {}", cycle, t, x_f32, y_f32, z_f32);
                            cycle += 1;
                            // println!("bytes: {:?}", data);
                        }
                        // println!("value: {}", x_f32);
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }

    // println!("{:?}", serial_buf);
}