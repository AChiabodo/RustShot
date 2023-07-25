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
    image: Option<Arc<ImageBuf>>,
    #[data(eq)]
    display: Option<u32>
}

fn twmp_widget() -> impl Widget<State> {
    Flex::column()
        .with_child(Container::new(Button::new("Click me").on_click(|ctx, data: &mut State, _env|{
            let mut temp = ctx.window().clone();
            temp.set_window_state(druid::WindowState::Minimized);
            
            let thread = thread::spawn(||{
                let temp = take_screenshot(Display::primary().expect("Couldn't find display")).expect("Couldn't take screenshot");
                return temp;
            });
            
            let screenshot = thread.join().expect("Failed to take the screenshot");
            
            let new_image_buf = ImageBuf::from_raw(screenshot.to_vec(), ImageFormat::Rgb, screenshot.width() as usize, screenshot.height() as usize);
            match &mut data.image {
                None => data.image = Some(Arc::new(new_image_buf)),
                Some(val) => *Arc::make_mut(val) = new_image_buf
            }
            temp.set_window_state(druid::WindowState::Maximized);
            }
        )))
        .with_flex_child(
            ViewSwitcher::new(
            |data: &State, _env| data.clone(),
            move |_, data: &State, _env| {
                if data.image.is_some() {
                    Box::new(
                        Image::new(data.image.as_ref().unwrap().as_ref().clone()).lens(State::image)
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