use scrap::Display;

mod screen;
mod gui;
fn main()  {
    let temp = screen::display_list();
    for (i,dspl) in temp.iter().enumerate(){
        println!("Display {} : {} x {}",i,dspl.height(),dspl.width());
    }
    //gui::main_window().unwrap();
    screen::take_screenshot("screenshot.png".to_string(), Display::primary().expect("Couldn't find primary display."));
}