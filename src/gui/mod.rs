use std::sync::Arc;

use druid::piet::ImageFormat;
use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc, Data, Lens};
use image::ImageBuffer;
use scrap::Display;

use crate::screen::take_screenshot;

use druid::{
    widget::{Image, FillStrat},
    piet::{ImageBuf, InterpolationMode},
};

#[derive(Clone, Data, Lens)]
struct AppData {
    #[data(eq)]
    image: Option<ImageBuffer<image::Rgb<u8>, Vec<u8>>>,
    #[data(eq)]
    display: Option<u32>
}

fn twmp_widget() -> impl Widget<AppData> {
    let image_data = ImageBuf::from_raw(
        vec![0; 100 * 100 * 4],
        ImageFormat::Rgb,
        100,
        100,
    );
    let image_widget = Image::new(image_data)
        // set the fill strategy
        .fill_mode(FillStrat::Fill)
        // set the interpolation mode
        .interpolation_mode(InterpolationMode::Bilinear);

    let button_screenshot = Button::new("Take Screenshot")
    .on_click( |_ctx, data, _env| -> (){

        *data = AppData{image : Some(take_screenshot(Display::primary().expect("Couldn't find display")).expect("Couldn't take screenshot")), display : None};
        //let (w,h) = img.dimensions();
        //image::save_buffer("screenshot.png".to_string(), &img, w, h, image::ColorType::Rgb8).unwrap();
    } );
    let button_save = Button::new("Save Screenshot")
    .on_click(|_ctx, data: &mut AppData, _env| -> () {
        if let Some(img) = &data.image {
            let (w,h) = img.dimensions();
            image::save_buffer("screenshot.png".to_string(), &img, w, h, image::ColorType::Rgb8).unwrap();
        }
    });
    Flex::column()
        .with_child(button_screenshot)
        .with_child(button_save)
        .with_child(image_widget)
}

fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current locale and count
    let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());

    let label = Label::new(text).padding(5.0).center();
    let button_increment = Button::new("increment")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);
    let button_decrement = Button::new("decrement")
        .on_click(
            |_ctx, data, _env| -> () {
            if *data <= 0 {
                *data = 0
            } else {
                *data -= 1
            }
        }
        )
        .padding(5.0);

    let button_screenshot = Button::new("Take Screenshot")
        .on_click( |_ctx, data : &mut u32, _env| -> (){
            let img = take_screenshot(Display::primary().expect("Couldn't find display")).expect("Couldn't take screenshot");
            let (w,h) = img.dimensions();
            image::save_buffer("screenshot.png".to_string(), &img, w, h, image::ColorType::Rgb8).unwrap();
        } )
        .padding(5.0);
    Flex::column()
        .with_child(label)
        .with_child(button_increment)
        .with_child(button_decrement)
        .with_child(button_screenshot)
}

pub fn main_window() -> Result<(), PlatformError>{
    let main_window = WindowDesc::new(twmp_widget());
    let data = AppData { image: None, display: None };
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
}