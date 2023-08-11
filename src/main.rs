mod screen;
mod gui;
fn main()  {
    //let temp = screen::display_list();
    //let i = screen::try_screenshot();
    gui::main_window().unwrap();
}