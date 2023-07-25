use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};
use scrap::Display;

use crate::screen::take_screenshot;

use druid::{
    widget::{Image, FillStrat},
    piet::{ImageBuf, InterpolationMode},
};

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

        let image_data = ImageBuf::empty();
        let image_widget = Image::new(image_data)
            // set the fill strategy
            .fill_mode(FillStrat::Fill)
            // set the interpolation mode
            .interpolation_mode(InterpolationMode::Bilinear);

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
        .with_child(image_widget)
}

pub fn main_window() -> Result<(), PlatformError>{
    let main_window = WindowDesc::new(ui_builder());
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
}