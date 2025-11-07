use crate::{api::device::Device, data_worker::particle::ParticleType};
use std::io::Write;

mod api;
mod data_worker;

fn main() {
    let handle = api::handle::PixHandle::new();
    println!("Device count: {}", handle.get_device_count());

    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(0.5)
        .threshold(10.0)
        .high_voltage(80.0);
    let device = handle.get_device(builder).unwrap();

    loop {
        let image_buffer = device.capture_image().unwrap();
        device.save_last_frame("output.png").unwrap();

        let image = image_buffer
            .chunks(device.get_dimensions().0 as usize)
            .map(|buf| buf.into_iter().cloned().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let mut frame = data_worker::frame::Frame::new(image);
        frame.count_particles(12);

        let mut frame_particles = frame.get_particles();
        for particle in &mut frame_particles {
            particle.calculate_type();
            match particle.particle_type {
                ParticleType::PossibleMuon(size) => {
                    println!("Found muon of size {}", size);
                    let mut log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("log.txt")
                        .unwrap();
                    log.write_all(format!("{:?}", frame).as_bytes()).unwrap();
                    log.flush().unwrap();
                    },
                _ => {},
            }
        }
    }
}
