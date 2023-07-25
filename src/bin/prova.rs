use druid::widget::{ViewSwitcher, FillStrat, Image, Svg, Flex};
use druid::Widget;
use druid::Lens;
use druid::Data;
use druid::AppLauncher;
use druid::WidgetExt;
use druid::WindowDesc;
use druid::ImageBuf;
use druid::widget::Button;
use std::sync::Arc;
use druid::EventCtx;
use druid::Env;


#[derive(Clone, Default, Lens, Data)]
pub struct State {
    pub display_image: Option<Arc<ImageBuf>>
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title("Test")
        .window_size((400.0, 600.0));

    // create the initial app state
    let initial_state = State {
        display_image: None
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<State> {
    Flex::column()
        .with_child(
            ViewSwitcher::new(
            |data: &State, _env| data.clone(),
            move |_, data: &State, _env| {
                if data.display_image.is_some() {
                    Box::new(
                        Image::new(data.display_image.as_ref().unwrap().as_ref().clone()).lens(State::display_image)
                    )
                }
                else {
                    Box::new(Image::new(ImageBuf::from_raw(vec!(0, 0, 0), druid::piet::ImageFormat::Rgb, 1, 1)))
                }
            },
        ))
        .with_spacer(50.0)
        .with_child(Button::new("Click me")).on_click(|_: &mut EventCtx, data: &mut State, _: &Env| {
            use rand::Rng;
            use druid::piet::ImageFormat;
            let new_image_buf = ImageBuf::from_raw(vec!(rand::thread_rng().gen(), rand::thread_rng().gen(), rand::thread_rng().gen()), ImageFormat::Rgb, 1, 1);
            match &mut data.display_image {
                None => data.display_image = Some(Arc::new(new_image_buf)),
                Some(val) => *Arc::make_mut(val) = new_image_buf
            }
        })
}