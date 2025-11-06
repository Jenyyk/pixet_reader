use crate::api::device::Device;

mod api;

fn main() {
    let handle = api::handle::PixHandle::new();
    println!("Device count: {}", handle.get_device_count());

    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(3.0)
        .threshold(100.0)
        .high_voltage(40.0);
    let device = handle.get_device(builder).unwrap();

    let image_buffer = device.capture_image().unwrap();

    let image = image_buffer.chunks(device.get_dimensions().0 as usize).map(|buf| buf.into_iter().cloned().collect::<Vec<_>>()).collect::<Vec<_>>();
}
