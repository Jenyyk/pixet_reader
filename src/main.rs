use crate::{api::device::Device, data_worker::particle::ParticleType};
use std::io::Write;

mod api;
mod data_worker;

fn main() {
    let handle = api::handle::PixHandle::new();
    println!("Device count: {}", handle.get_device_count());

    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(0.5)
        .threshold(10.0);

    let device = handle.get_device(builder).unwrap();

    let max_voltage = match device.get_voltage_range() {
        Ok((_min, max)) => max,
        Err(_) => 80.0,
    };
    println!("Applying max voltage of {}V", max_voltage);
    device.set_high_voltage(max_voltage).unwrap();

    let mut muons_found = 0;
    loop {
        let image_buffer = device.capture_image().unwrap();

        let image = image_buffer
            .chunks(device.get_dimensions().0 as usize)
            .map(|buf| buf.to_vec())
            .collect::<Vec<_>>();

        let mut frame = data_worker::frame::Frame::new(image);
        frame.count_particles(12);

        let mut frame_particles = frame.get_particles();
        for particle in &mut frame_particles {
            particle.calculate_type();
            match particle.particle_type {
                ParticleType::PossibleMuon(size) => {
                    println!("Found muon of size {}", size);
                    device
                        .save_last_frame(format!("muon{}.png", muons_found))
                        .unwrap();
                    muons_found += 1;
                    let mut log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("log.txt")
                        .unwrap();
                    log.write_all(format!("{:?}", frame).as_bytes()).unwrap();
                    log.flush().unwrap();
                }
                _ => {}
            }
        }
    }
}
