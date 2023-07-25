use std::{sync::Arc, thread, time::Duration};

use druid::{AppLauncher, PlatformError, Widget, WidgetExt, WindowDesc, Data, Lens, widget::{FlexParams, CrossAxisAlignment}};
use scrap::Display;
use druid::{
    WindowHandle,
    widget::{Button, Flex, Container, ViewSwitcher,Image},
    piet::{ImageBuf,ImageFormat},
};
use crate::screen::take_screenshot;

#[derive(Clone, Data, Lens)]
struct State {
    image: Option<ImageBuf>,
    #[data(eq)]
    display: Option<u32>
}

fn twmp_widget() -> impl Widget<State> {
    Flex::column()
        .with_child(Flex::row().with_child(Button::new("Take screenshot").on_click(|_ctx, data: &mut State, _env|{
            let screenshot = take_screenshot(Display::primary().expect("Couldn't find display")).expect("Couldn't take screenshot");
            let new_image_buf = ImageBuf::from_raw(screenshot.to_vec(), ImageFormat::Rgb, screenshot.width() as usize, screenshot.height() as usize);
            match &mut data.image {
                None => data.image = Some(new_image_buf),
                Some(val) => *val = new_image_buf
            }
            }
        )).with_child(Button::new("Save image")).on_click(|_ctx, data: &mut State, _env| {
            match &data.image {
                None => (),
                Some(val) => match image::save_buffer("./screen.png", val.raw_pixels(), val.width() as u32, val.height() as u32, image::ColorType::Rgb8) {
                    Ok(_) => (),
                    Err(err) => println!("{}", err),
                },
            }
        }))
        .with_flex_child(
            ViewSwitcher::new(
            |data: &State, _env| data.clone(),
            move |_, data: &State, _env| {
                if data.image.is_some() {
                    Box::new(
                        Image::new(data.image.as_ref().unwrap().clone()).lens(State::image)
                    )
                }
                else {
                    Box::new(Image::new(ImageBuf::from_raw(vec!(0, 0, 0), druid::piet::ImageFormat::Rgb, 1, 1)))
                }
            },
        ),FlexParams::new(1.0, CrossAxisAlignment::Fill)
        )
}

pub fn main_window() -> Result<(), PlatformError>{
    let main_window = WindowDesc::new(twmp_widget());
    let data = State { image: None, display: None };
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
}