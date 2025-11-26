use crate::{
    api::device::Device,
    api::ffi::PxcIgnoreErr,
    data_worker::{
        frame::Frame,
        particle::{Particle, ParticleType},
    },
};
use std::{io::Write, path::Path};

mod api;
mod data_worker;
mod library;

struct ArgOptions {
    pub save_mode: SaveMode,
    pub filter: Box<dyn Fn(&Particle) -> bool>,
    pub save_images: bool,
    pub thresholds: (f64, f64, f64),
}

const THRESHOLD_MIN_DEFAULT: f64 = 0.0;
const THRESHOLD_MAX_DEFAULT: f64 = 0.0;
const THRESHOLD_PIX_DEFAULT: f64 = 0.2;
const HIGH_VOLTAGE_DEFAULT: f64 = 50.0;
const FRAME_TIME_DEFAULT: f64 = 2.0;

fn main() -> () {
    let mut standalone = false;
    let mut save_mode = SaveMode::AlmostJson;
    let mut filter: Box<dyn Fn(&Particle) -> bool> = Box::new(|_particle| true);
    let mut save_images = false;
    let mut threshold_pix = THRESHOLD_PIX_DEFAULT;
    let mut threshold_min = THRESHOLD_MIN_DEFAULT;
    let mut threshold_max = THRESHOLD_MAX_DEFAULT;

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
            "--filter" | "-F" => {
                filter = make_filter(
                    args.next()
                        .expect("Empty flag set for --filter")
                        .to_ascii_lowercase()
                        .split(",")
                        .map(|str| str.to_owned())
                        .collect::<Vec<String>>(),
                )
            }
            "--save-images" | "-I" => save_images = true,
            "--threshold-pix" => {
                threshold_pix = args
                    .next()
                    .expect("Empty flag set for --threshold-pix")
                    .parse::<f64>()
                    .expect("Invalid flag set for --threshold-pix");
            }
            "--threshold-min" => {
                threshold_min = args
                    .next()
                    .expect("Empty flag set for --threshold")
                    .parse::<f64>()
                    .expect("Invalid flag set for --threshold");
            }
            "--threshold-max" => {
                threshold_max = args
                    .next()
                    .expect("Empty flag set for --threshold")
                    .parse::<f64>()
                    .expect("Invalid flag set for --threshold");
            }
            _ => eprintln!("Invalid flag: '{}'", arg),
        }
    }

    if standalone {
        let arg_options: ArgOptions = ArgOptions {
            save_mode,
            filter,
            save_images,
            thresholds: (threshold_min, threshold_max, threshold_pix),
        };
        start_standalone_reader(arg_options);
        return;
    } else {
        library::start_library();
        return;
    }
}

fn start_standalone_reader(options: ArgOptions) {
    let handle = api::handle::PixHandle::new();
    println!("[info]Device count: {}", handle.get_device_count());

    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(0.5)
        .hardware_threshold(options.thresholds.2);

    let mut device = handle.get_device(builder).unwrap();

    let max_voltage = match device.get_voltage_range() {
        Ok((_min, max)) => max,
        Err(_) => 80.0,
    };
    println!("[info]Found max voltage of {}V", max_voltage);
    device.set_high_voltage(HIGH_VOLTAGE_DEFAULT).ignore_error();

    device.set_software_high_threshold(options.thresholds.1);
    device.set_software_low_threshold(options.thresholds.0);

    let mut particles_found = 0;
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
        // this is the kernel size
        // yes it is a magic number
        // no i do not care
        frame.count_particles(12);

        for particle in frame.get_particles_mut() {
            particle.calculate_type();
        }

        for particle in frame.get_particles() {
            if (options.filter)(&particle) {
                println!(
                    "[info]{}Found particle {:?}",
                    particles_found, particle.particle_type
                );
                save_frame("log.txt", frame.clone(), options.save_mode).unwrap();
                if options.save_images
                    && let Err(why) = device.save_last_frame(&format!(
                        "particle{particles_found}{:?}.png",
                        &particle.particle_type
                    ))
                {
                    eprintln!("[err]Failed to save frame: {why:?}");
                }
                particles_found += 1;
                break;
            }
        }
    }
}

#[derive(Clone, Copy)]
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
            file.write_all(b"----------\n")?; // 10 pomlček
        }
    }
    file.flush()?;

    Ok(())
}

#[allow(clippy::single_match)]
fn make_filter(to_filter: Vec<String>) -> Box<dyn Fn(&Particle) -> bool> {
    Box::new(move |particle| {
        if to_filter.is_empty() {
            return true;
        }
        for t in &to_filter {
            match t.as_str() {
                "muon" => {
                    if matches!(particle.particle_type, ParticleType::PossibleMuon(_)) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_work() {
        let filter = make_filter(vec![String::from("muon")]);
        let muon = Particle {
            particle_type: ParticleType::PossibleMuon(5),
            positions: vec![],
        };
        let diff_particle = Particle {
            particle_type: ParticleType::Unknown,
            positions: vec![],
        };

        assert!(filter(&muon));
        assert!(!filter(&diff_particle));

        let filter = make_filter(vec![]);

        assert!(filter(&muon));
        assert!(filter(&diff_particle));
    }
}
