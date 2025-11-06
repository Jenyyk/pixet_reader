# Pixet Reader

This is a binary template for rust projects that read data from [Advacam Radiation imagers](https://advacam.com/)

## Quickstart
All needed libraries for specific distributions are included in the repo
Here is a simple showcase of how to use the template:

```rust
mod api;

fn main() {
    // The dynamic library must be initialized by creating a handle
    // When this handle is dropped it also deinitializes the library,
    // So only ever create one handle
    let handle = api::handle::PixHandle::new();

    // Creates a device builder
    // takes in index as input (devices are indexed from 0)
    //
    // All fields are optional and have reasonable defaults if left unset
    let builder = api::handle::DeviceBuilder::new(0)
        .frame_time(5.0)
        .threshold(100.0)
        .high_voltage(40.0);

    // Creates the actual Device struct from the handle
    let device = handle.get_device(builder).unwrap();

    // Captures a frame into the devices memory and also returns it
    let _image = device.capture_image().unwrap();

    // Saves the current frame saved in memory into a file
    device.save_last_frame("output.png").unwrap()
}
```
