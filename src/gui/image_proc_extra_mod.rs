use eframe::egui::{Pos2, Rect, Vec2};
use image::{DynamicImage, GenericImageView, Pixel};
use imageproc::drawing;
use imageproc::drawing::{BresenhamLineIter, Canvas};

pub fn draw_arrow (img: &mut DynamicImage, start:(f32, f32), end:(f32, f32), t: usize, color: [u8; 4]) {
    draw_thick_line(img, start, end, t, color);
    let v = Vec2::new(end.0 - start.0, end.1 - start.1).normalized();
    let perpendicular = v.rot90().normalized();
    let t2 = (t as f32)/3. + 1.;
    let p1 = (end.0 - v.x * 10.0 * t2  - perpendicular.x * 10.0 * t2 , end.1 - v.y * 10.0 * t2 - perpendicular.y * 10.0 * t2);
    let p2 = (end.0 - v.x * 10.0 * t2  + perpendicular.x * 10.0 * t2 , end.1 - v.y * 10.0 * t2 + perpendicular.y * 10.0 * t2);
    draw_thick_line(img, p1, end, t, color);
    draw_thick_line(img, p2, end, t, color);
}

pub fn draw_thick_line(img: &mut DynamicImage, start:(f32, f32), end:(f32, f32), t: usize, color: [u8; 4]) {
    let segment = bresenham_line(start.0 as usize, start.1 as usize, end.0 as usize, end.1 as usize);
    for point in segment {
        drawing::draw_filled_circle_mut(img, (point.0 as i32, point.1 as i32), t as i32, color.into());
    }
}

pub fn erase_thick_line(original_img:&DynamicImage, img: &mut DynamicImage, start:(f32, f32), end:(f32, f32), t: usize) {
    let segment = bresenham_line(start.0 as usize, start.1 as usize, end.0 as usize, end.1 as usize);
    for point in segment {
        erase_filled_circle_mut(original_img, img, (point.0 as i32, point.1 as i32), t as i32);
    }
}

pub fn draw_blended_line_segment_mut(original_canvas:&DynamicImage, canvas: &mut DynamicImage, start: (f32, f32), end: (f32, f32), mut color: [u8; 4]) {
    let (width, height) = GenericImageView::dimensions(canvas);
    let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

    let line_iterator = BresenhamLineIter::new(start, end);

    for point in line_iterator {
        let x = point.0;
        let y = point.1;

        if in_bounds(x, y) {
            let mut pixel = GenericImageView::get_pixel(original_canvas, x as u32, y as u32);
            //Make the alpha channel equals half of the alpha channel of the original pixel
            color[3] = pixel[3] / 10;
            pixel.blend(&color.into());
            canvas.draw_pixel(x as u32, y as u32, pixel);
        }
    }
}

pub fn draw_blended_filled_circle_mut(original_canvas:&DynamicImage, canvas: &mut DynamicImage, center: (i32, i32), radius: i32, color: [u8; 4]) {
    let mut x = 0i32;
    let mut y = radius;
    let mut p = 1 - radius;
    let x0 = center.0;
    let y0 = center.1;

    while x <= y {
        draw_blended_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
            color,
        );
        draw_blended_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
            color,
        );
        draw_blended_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
            color,
        );
        draw_blended_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

///Erase a segment from [canvas], restoring the original pixels of [original_canvas]
pub fn erase_line_segment_mut(original_canvas:&DynamicImage, canvas: &mut DynamicImage, start: (f32, f32), end: (f32, f32)) {
    let (width, height) = GenericImageView::dimensions(canvas);
    let in_bounds = |x, y| x >= 0 && x < width as i32 && y >= 0 && y < height as i32;

    let line_iterator = BresenhamLineIter::new(start, end);

    for point in line_iterator {
        let x = point.0;
        let y = point.1;

        if in_bounds(x, y) {
            // Get the original pixel
            let mut pixel = GenericImageView::get_pixel(original_canvas, x as u32, y as u32);
            canvas.draw_pixel(x as u32, y as u32, pixel);
        }
    }
}

///Erase a circle from [canvas], restoring the original pixels of [original_canvas]
pub fn erase_filled_circle_mut(original_canvas:&DynamicImage, canvas: &mut DynamicImage, center: (i32, i32), radius: i32) {
    let mut x = 0i32;
    let mut y = radius;
    let mut p = 1 - radius;
    let x0 = center.0;
    let y0 = center.1;

    while x <= y {
        erase_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
        );
        erase_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
        );
        erase_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
        );
        erase_line_segment_mut(
            original_canvas,
            canvas,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

pub fn highlight_line(original_img:&DynamicImage, img: &mut DynamicImage, start:(f32, f32), end:(f32, f32), t: usize, mut color: [u8; 4]) {
    let segment = bresenham_line(start.0 as usize, start.1 as usize, end.0 as usize, end.1 as usize);
    for point in segment {
        draw_blended_filled_circle_mut(original_img,img, (point.0 as i32, point.1 as i32), t as i32, color.into());
    }
}

pub fn bresenham_line(x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<(usize, usize)> {
    let mut points = Vec::new();

    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();

    let mut x = x0 as i32;
    let mut y = y0 as i32;

    let x_inc = if x1 > x0 { 1 } else { -1 };
    let y_inc = if y1 > y0 { 1 } else { -1 };

    let mut error = dx - dy;

    while x != x1 as i32 || y != y1 as i32 {
        points.push((x as usize, y as usize));

        let error2 = error * 2;

        if error2 > -dy {
            error -= dy;
            x += x_inc;
        }

        if error2 < dx {
            error += dx;
            y += y_inc;
        }
    }

    points.push((x1, y1)); // Include the endpoint

    points
}

/// Transform the absolute position ([Pos2]) of the mouse on the application window into a relative position with respect to the given [Rect]
///
/// [Rect] must be meaningful with respect to the application window. (It needs to actually be a part of the application window to obtain a meaningful relative position)
pub fn into_relative_pos(pos: Pos2, rect: Rect) -> Pos2 {
    Pos2::new(pos.x - rect.left(), pos.y - rect.top())
}