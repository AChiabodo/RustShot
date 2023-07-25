use scrap::{Capturer, Display};
use std::fs::File;
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

use image::{DynamicImage,ImageBuffer};

pub fn display_list() -> Vec<Display> {
    let temp = Display::all().expect("Can't find any screen");
    return temp;
}

pub fn take_screenshot(display: Display) -> Option<ImageBuffer<image::Rgb<u8>, Vec<u8>>> {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        // Wait until there's a frame.

        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        println!("Captured! Saving...");
        let stride = buffer.len() / h;

        let img = ImageBuffer::from_fn(w as u32,h as u32, |x, y| {
            let i = stride * y as usize + 4 * x as usize;
            image::Rgb([buffer[i + 2], buffer[i + 1], buffer[i]]) //flip the bits
        });
        // Save the image.
        //let temp = img.into_raw();
        //repng::encode(File::create(path).unwrap(), w as u32, h as u32, &bitflipped).unwrap();
        //image::save_buffer(path, &buffer, w as u32, h as u32, image::ColorType::Rgba8).unwrap();
        break Some(img);
    }
}
