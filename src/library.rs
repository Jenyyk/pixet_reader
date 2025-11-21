use crate::api::ffi::PxcIgnoreErr;
use crate::api::{device::Device, handle::DeviceBuilder};
use crate::data_worker::frame::Frame;
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::sync::{Arc, RwLock};
use std::thread;

struct DeviceHolder {
    device: Arc<RwLock<Box<dyn Device>>>,
    buffer_queue: Arc<RwLock<Vec<Frame>>>,
}

pub fn start_library() {
    let mut devices: HashMap<u32, DeviceHolder> = HashMap::new();
    let stdin = std::io::stdin();
    let handle = crate::api::handle::PixHandle::new();
    loop {
        let mut input = String::new();
        if let Err(why) = stdin.read_line(&mut input) {
            eprintln!("Error reading input: {}", why);
        }
        let mut command = input.split_whitespace();
        while let Some(arg) = command.next() {
            match arg {
                "add" => {
                    let index: u32 = command.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
                    if !devices.contains_key(&index) {
                        let builder = DeviceBuilder::new(index);
                        let device = match handle.get_device(builder) {
                            Ok(device) => device,
                            Err(why) => {
                                eprintln!("[err]Failed to get device: {why:?}");
                                break;
                            }
                        };
                        devices.insert(
                            index,
                            DeviceHolder {
                                device: Arc::new(RwLock::new(Box::new(device))),
                                buffer_queue: Arc::new(RwLock::new(Vec::new())),
                            },
                        );
                    }
                    let device_holder = devices.get(&index).unwrap();

                    let device_clone = device_holder.device.clone();
                    let buffer_clone = device_holder.buffer_queue.clone();

                    // spawn thread to capture data of device
                    thread::spawn(move || start_dev_loop(device_clone, buffer_clone));

                    set_device_settings(device_holder, command);
                    break;
                }
                "set" => {
                    let index: u32 = command.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
                    let device_holder = match devices.get(&index) {
                        Some(holder) => holder,
                        None => {
                            eprintln!("[err]Device not created");
                            break;
                        }
                    };
                    set_device_settings(device_holder, command);
                    break;
                }
                "get" => {
                    let index: u32 = command.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
                    match devices.get_mut(&index) {
                        Some(holder) => {
                            print_buffers(holder);
                        }
                        None => eprintln!("[err]Device not created"),
                    }
                    break;
                }
                _ => {}
            }
        }
    }
}

fn set_device_settings<'a>(holder: &DeviceHolder, mut command: impl Iterator<Item = &'a str>) {
    let device_clone = holder.device.clone();
    while let Some(arg) = command.next() {
        match arg {
            "frame-time" => {
                let mut device = device_clone.write().unwrap();
                device.set_frame_time(command.next().unwrap_or("2.0").parse::<f64>().unwrap_or(2.0)).ignore_error();
            },
            _ => {},
        }
    }
}

fn start_dev_loop(device: Arc<RwLock<Box<dyn Device>>>, buffer: Arc<RwLock<Vec<Frame>>>) {
    loop {
        let device = device.read().unwrap();
        let out_buf = device.capture_image().unwrap();
        let image = out_buf
            .chunks(device.get_dimensions().0 as usize)
            .map(|buf| buf.to_vec())
            .collect::<Vec<_>>();
        let frame = Frame::new(image);

        let mut buf_mut = buffer.write().unwrap();
        buf_mut.push(frame);
    }
}

fn print_buffers(holder: &mut DeviceHolder) {
    let buffer_queue_clone = holder.buffer_queue.clone();
    let buffer_queue = match buffer_queue_clone.read() {
        Ok(queue) => queue,
        Err(why) => {
            eprintln!("[err]Failed to read buffer queue: {why:?}");
            return;
        }
    };
    let len = buffer_queue.len();

    let mut stdout = stdout().lock();

    // print number of frames that can be expected
    writeln!(stdout, "[len]{}", len).unwrap();

    for frame in buffer_queue.iter() {
        writeln!(stdout, "[frame]{:?}", frame).unwrap();
    }
}
