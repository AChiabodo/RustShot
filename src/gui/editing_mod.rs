use crate::gui::image_proc_extra_mod::*;
use std::cmp::max;
use std::collections::VecDeque;
use std::io::Cursor;
use eframe::egui::Pos2;
use egui::Rect;
use image::DynamicImage;
use imageproc::drawing;
use png::Decoder;
use rusttype::Font;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Tool {
    Drawing,
    HollowRect,
    FilledRect,
    Arrow,
    HollowCircle,
    FilledCircle,
    Eraser,
    Crop,
    Highlighter,
    Text,
    None,
}

#[derive(Clone)]
pub struct TextManager {
    pub text_areas: Vec<TextManager>,
    pub curr_font: Option<Font<'static>>,
    pub curr_str: String,
    pub writing: bool,
    pub edge: Pos2,
    pub width: f32,
    pub height: f32,
    pub curr_dim: i32,
    pub max_width: f32,
    pub max_height: f32,
    pub dirty: bool,
    //Needed since i rewrite everytime all the text on the screen during editing
    pub original_img: Image,
    pub curr_font_name: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl TextManager {
    pub fn new (_font: String, edge: Pos2, img: Image) -> TextManager {
        let font_bytes = include_bytes!("../../resources/Roboto-Regular.ttf");
        let font = Font::try_from_bytes(font_bytes);
        TextManager{
            text_areas: Vec::new(),
            curr_font: font,
            curr_font_name: "Roboto".to_string(),
            curr_str: "".to_string(),
            curr_dim: 15,
            writing: false,
            edge,
            max_width:0.,
            width:0.,
            height:0.,
            dirty: false,
            original_img: img,
            cursor_x:0,
            cursor_y:0,
            max_height: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.text_areas.clear();
        self.curr_str = "".to_string();
        self.curr_dim = 15;
        self.writing = false;
        self.max_width = 0.;
        self.max_height = 0.;
        self.width = 0.;
        self.height = 0.;
        self.dirty = false;
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn update_max_width(&mut self, rect: Rect) {
        self.max_width = rect.right() - self.edge.x;
    }

    pub fn update_max_height(&mut self, rect: Rect) {
        self.max_height = rect.bottom() - self.edge.y;
    }
}


pub struct PaintState {
    pub curr_tool: Tool,
    pub curr_color: [u8; 4],
    pub curr_thickness: usize,
    pub text_info: TextManager,
    pub painting: bool,
    pub last_ptr: Pos2,
    pub curr_ptr: Pos2,
}

impl PaintState {

    pub fn new()-> Self {
        PaintState {
            curr_tool: Tool::None,
            curr_color: [255, 255, 255, 255],
            curr_thickness: 1,
            painting: false,
            text_info: TextManager::new("Roboto-Light".to_string(), Pos2::default(), Image::new(DynamicImage::default(), 0)),
            last_ptr: Pos2::default(),
            curr_ptr: Pos2::default(),
        }
    }
    /// Reset the paint state to its default values, including the current tool and color
    pub fn reset(&mut self) {
        self.painting = false;
        self.last_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
        self.curr_tool = Tool::None;
        self.curr_color = [255, 255, 255, 255];
    }

    /// Reset the paint state to its default values, excluding the current tool and color
    pub fn soft_reset(&mut self) {
        self.painting = false;
        self.last_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
    }

    ///Draw a shape on the given img based on the field inside [self] ([curr_tool], [curr_color], [last_ptr], [curr_ptr], [curr_font], [curr_str])
    pub fn apply_tool(&mut self, img: &mut Image, original_img: Option<DynamicImage>) {
        let mut start_ptr = self.last_ptr;
        let width = max(1, (self.curr_ptr.x - self.last_ptr.x).abs() as i32);
        let height = max(1, (self.curr_ptr.y - self.last_ptr.y).abs() as i32);
        if self.curr_tool != Tool::Drawing && self.curr_tool != Tool::FilledCircle && self.curr_tool != Tool::HollowCircle {
            //Permits an easier selection, allowing to generate the area in all directions
            if self.curr_ptr.x < self.last_ptr.x {
                start_ptr.x = self.curr_ptr.x;
            }
            if self.curr_ptr.y < self.last_ptr.y {
                start_ptr.y = self.curr_ptr.y;
            }
        }
        match self.curr_tool {
            Tool::Drawing => {
                draw_thick_line(&mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::HollowRect => {
                draw_thick_hollow_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_thickness, self.curr_color.into());
            }
            Tool::FilledRect => {
                drawing::draw_filled_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::HollowCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                draw_thick_hollow_circle_mut(&mut img.image, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_thickness, self.curr_color.into());
            }
            Tool::FilledCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_filled_circle_mut(&mut img.image, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::Arrow => {
                draw_arrow(&mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Highlighter => {
                highlight_line(original_img.as_ref().unwrap(), &mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Crop => {
                let color = [0u8, 0u8, 0u8, 0u8];
                drawing::draw_hollow_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), color.into());
            }
            Tool::Eraser => {
                erase_thick_line(original_img.as_ref().unwrap(), &mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness);
            }
            Tool::Text => {
                //Unwrap cannot panic, text mode is allowed only if font loaded correctly
                let lines:Vec<&str> = self.text_info.curr_str.split("\n").collect();
                let mut y = self.text_info.edge.y;
                for l in lines{
                    drawing::draw_text_mut(&mut img.image, self.curr_color.into(), self.text_info.edge.x as i32, y as i32, rusttype::Scale::uniform(self.text_info.curr_dim as f32), self.text_info.curr_font.as_ref().unwrap(), l);
                    y += self.text_info.curr_dim as f32;
                }


            }
            _ => {}
        }
    }
}


#[derive(Clone)]
pub struct Image {
    image: DynamicImage,
    crop_index: usize,
}

impl Image {
    pub fn new(image: DynamicImage, crop_index: usize) -> Self {
        Image { image, crop_index }
    }
    pub fn get_image(&self) -> DynamicImage {
        self.image.clone()
    }

    pub fn get_crop_index(&self) -> usize {
        self.crop_index
    }

    pub fn get_width (&self) -> u32 {
        self.image.width()
    }

    pub fn get_height (&self) -> u32 {
        self.image.height()
    }
}

#[derive(Clone)]
pub struct CompressedImage {
    image: Vec<u8>,
    crop_index: usize,
}

impl CompressedImage {
    pub fn new(image: DynamicImage, crop_index: usize) -> Self {
        let rgba_data = image.as_bytes();
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), image.width(), image.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&rgba_data).unwrap();
        }
        CompressedImage { image: png_data, crop_index }
    }
    pub fn get_decompressed_image(&self) -> DynamicImage {
        let png_data = self.image.clone();
        // Create a decoder for the PNG data
        let cursor = Cursor::new(png_data);
        let decoder = Decoder::new(cursor);

        // Read the PNG data and decode it
        let mut reader = decoder.read_info().unwrap();
        let mut image_data = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut image_data).unwrap();

        // Convert the decoded image data to a DynamicImage
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
            reader.info().width,
            reader.info().height,
            image_data,
        ).unwrap());
        return image;
    }

    pub fn get_crop_index(&self) -> usize {
        self.crop_index
    }
}

impl Into<CompressedImage> for Image {
    fn into(self) -> CompressedImage {
        CompressedImage::new(self.image, self.crop_index)
    }
}

impl Into<Image> for CompressedImage {
    fn into(self) -> Image {
        Image::new(self.get_decompressed_image(), self.get_crop_index())
    }
}

pub struct ImageStack {
    images: VecDeque<CompressedImage>,
    redo_images: VecDeque<CompressedImage>,
    crop_images: Vec<DynamicImage>,
    pub tmp_image: Image,
    final_image: Image,
}

impl ImageStack {
    pub fn new(image: DynamicImage) -> Self {
        let mut images = VecDeque::new();
        images.push_front(CompressedImage::new(image.clone(), 0));
        let mut crop_images = Vec::new();
        crop_images.push(image.clone());
        ImageStack {
            images,
            redo_images: VecDeque::new(),
            crop_images,
            tmp_image: Image::new(image.clone(), 0),
            final_image: Image::new(image.clone(), 0),
        }
    }

    pub fn push_crop_image(&mut self, image: DynamicImage) {
        self.crop_images.push(image);
    }

    pub fn get_crop_image(&self, index:usize) -> DynamicImage {
        self.crop_images[index].clone()
    }

    pub fn get_crop_images_len(&self) -> usize {
        self.crop_images.len()
    }

    /// Push a new image to the redo_images stack
    pub fn push_redo_image(&mut self, image: Image) {
        self.redo_images.push_front(image.into());
    }

    /// Push an image from the redo_images stack
    pub fn pop_redo_image(&mut self) -> Option<Image> {
        match self.redo_images.pop_front(){
            None => {None}
            Some(img) => {
                Some(img.into())
            }
        }
    }

    pub fn get_images_len(&self) -> usize {
        self.images.len()
    }

    pub fn get_redo_images_len(&self) -> usize {
        self.redo_images.len()
    }

    /// Pop the last stacked image in the image stack, removing it. Returns [final image] if the stack is empty
    pub fn pop_last_image(&mut self) -> Image {
        match self.images.pop_front() {
            Some(img) => img.into(),
            None => self.final_image.clone()
        }
    }

    ///Get the width of the final image
    pub fn _get_width(&self) -> usize {
        return self.final_image.image.width() as usize;
    }

    ///Get the height of the final image
    pub fn _get_height(&self) -> usize {
        return self.final_image.image.height() as usize;
    }

    /// Get the last stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_last_image(&self) -> Image {
        match self.images.front() {
            Some(img) => (*img).clone().into(),
            None => self.final_image.clone()
        }
    }

    pub fn get_last_image_as_ref (&self) -> &CompressedImage {
        self.images.front().unwrap()
    }

    /// Get the first stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_first_image(&self) -> Image {
        match self.images.back() {
            Some(img) => (*img).clone().into(),
            None => self.final_image.clone()
        }
    }

    pub fn get_tmp_image(&self) -> Image {
        return self.tmp_image.clone();
    }

    pub fn set_tmp_image(&mut self, image: Image) {
        self.tmp_image = image.clone();
    }

    /// Stack an image in the image stack
    pub fn stack_image(&mut self, image: Image) {
        self.images.push_front(image.into());
    }

    /// Restore the image_stack to the first stacked image, or to [final image] if the stack is empty
    pub fn restore(&mut self) {
        let img = self.images.pop_back().unwrap_or(self.final_image.clone().into());
        self.images.clear();
        self.images.push_front(img.clone());
        self.tmp_image = img.clone().into();
        self.final_image = self.tmp_image.clone();
    }
    /// Clear the image stack
    pub fn _clear_stack(&mut self) {
        self.images.clear();
    }

    /// Save all changes made based on the current tmp image (the one the is actually shown on the app in edit mode)
    pub fn save_changes(&mut self) {
        // Reset all the crop_images and the crop_index for the saved image
        self.crop_images.clear();
        self.final_image = self.get_last_image();
        self.tmp_image = self.get_last_image();
        self.final_image.crop_index = 0;
        self.tmp_image.crop_index = 0;
        self.crop_images.push(self.final_image.get_image());
        self.images.clear();
        self.redo_images.clear();
        self.stack_image(self.final_image.clone());

    }

    pub fn undo_changes(&mut self) {
        // Reset all the crop_images and the crop_index for the saved image
        self.crop_images.clear();
        self.final_image = self.get_first_image();
        self.tmp_image = self.get_first_image();
        self.final_image.crop_index = 0;
        self.tmp_image.crop_index = 0;
        self.crop_images.push(self.final_image.get_image());
        self.images.clear();
        self.redo_images.clear();
        self.stack_image(self.final_image.clone());
    }

    /// Get the final image
    pub fn get_final_image(&self) -> Image {
        self.final_image.clone()
    }

    pub fn _set_final_image(&mut self, image: Image) {
        self.final_image = image;
    }
}