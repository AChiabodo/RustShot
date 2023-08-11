use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

use screenshots::{DisplayInfo, Image, Screen};

use image::{DynamicImage, ImageBuffer};

pub fn display_list() -> Vec<DisplayInfo> {
    let temp = screenshots::DisplayInfo::all().unwrap();
    return temp;
}

pub fn take_screenshot(disp: &DisplayInfo) -> Option<DynamicImage> {
    let tk : Screen = Screen::new(disp);
    let capture = tk.capture();
    let screen;
    match capture 
    {
        Ok(capture)=> screen = capture,

        Err(_) => return None,

    }
    let im = screen.rgba();
    let (w, h) = (screen.width(), screen.height());
    let stride = im.len() / h as usize;
    let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
            let i = stride * y as usize + 4 * x as usize;
            image::Rgba([im[i], im[i + 1], im[i + 2], im[i + 3]])
        });
    let img2 = DynamicImage::from(img);
    return Some(img2);
    }

