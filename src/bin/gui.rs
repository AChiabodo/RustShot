use druid::widget::{Button, Flex, Label};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder());
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
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

    Flex::column()
        .with_child(label)
        .with_child(button_increment)
        .with_child(button_decrement)
}