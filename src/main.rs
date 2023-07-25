use scrap::Display;

mod screen;
mod gui;
fn main()  {
    let temp = screen::display_list();
    for (i,dspl) in temp.iter().enumerate(){
        println!("Display {} : {} x {}",i,dspl.height(),dspl.width());
    }
    //gui::main_window().unwrap();
    //screen::take_screenshot("screenshot.png".to_string(), Display::primary().expect("Couldn't find primary display."));
    for (i,display) in Display::all().unwrap().into_iter().enumerate() {
        let img = screen::take_screenshot(display).expect("Failed to take the screenshot");
        let (w,h) = img.dimensions();
        image::save_buffer(format!("screenshot_{}.png",i), &img, w, h, image::ColorType::Rgb8).unwrap();
    }
    gui::main_window().unwrap();
}