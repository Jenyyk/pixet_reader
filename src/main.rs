use crate::{
    api::device::Device,
    api::ffi::PxcIgnoreErr,
    data_worker::{frame::Frame, particle::ParticleType},
};
use std::{io::Write, path::Path};

mod api;
mod data_worker;

struct ArgOptions {
    pub save_mode: SaveMode,
}

fn main() {
    let mut standalone = false;
    let mut save_mode = SaveMode::AlmostJson;

    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--standalone" | "-S" => standalone = true,
            "--save-mode" | "-M" => match args
                .next()
                .expect("Empty flag set for --save-mode")
                .to_ascii_lowercase()
                .as_str()
            {
                "json" => save_mode = SaveMode::AlmostJson,
                "rak" => save_mode = SaveMode::RawRakMatrix,
                _ => panic!("Invalid flag set for --save-mode"),
            },
            _ => {}
        }
    }

    let arg_options: ArgOptions = ArgOptions { save_mode };

    if standalone {
        start_standalone_reader(arg_options);
        std::process::exit(0);
    }
}

fn start_standalone_reader(options: ArgOptions) {
    let handle = api::handle::PixHandle::new();
    println!("[info]Device count: {}", handle.get_device_count());

    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(0.5)
        .threshold(0.2);

    let device = handle.get_device(builder).unwrap();

    let max_voltage = match device.get_voltage_range() {
        Ok((_min, max)) => max,
        Err(_) => 80.0,
    };
    println!("[info]Found max voltage of {}V", max_voltage);
    device.set_high_voltage(50.0).ignore_error();

    let mut muons_found = 0;
    loop {
        let image_buffer = device.capture_image();
        if image_buffer.is_err() {
            image_buffer.ignore_error();
            continue;
        }
        let image_buffer = image_buffer.unwrap();

        let image = image_buffer
            .chunks(device.get_dimensions().0 as usize)
            .map(|buf| buf.to_vec())
            .collect::<Vec<_>>();

        let mut frame = data_worker::frame::Frame::new(image);
        frame.count_particles(12);

        for particle in frame.get_particles_mut() {
            particle.calculate_type();
        }

        for particle in frame.get_particles() {
            match particle.particle_type {
                ParticleType::PossibleMuon(size) => {
                    println!("[info]{}: Found muon of size {}", muons_found, size);
                    device
                        .save_last_frame(format!("muon{}.png", muons_found))
                        .unwrap();
                    muons_found += 1;
                    save_frame("log.txt", frame, options.save_mode.clone()).unwrap();
                    break;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
enum SaveMode {
    AlmostJson,
    RawRakMatrix,
}
fn save_frame(path: impl AsRef<Path>, frame: Frame, mode: SaveMode) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    match mode {
        SaveMode::AlmostJson => {
            file.write_all(format!("{:?}", frame).as_bytes())?;
            file.write_all(b"\n")?;
        }
        SaveMode::RawRakMatrix => {
            for row in frame.data {
                let string = row
                    .iter()
                    .map(|&x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                file.write_all(string.as_bytes())?;
                file.write_all(b"\n")?;
            }
            file.write_all(b"----------")?; // 10 pomlček
        }
    }
    file.flush()?;

    Ok(())
}
