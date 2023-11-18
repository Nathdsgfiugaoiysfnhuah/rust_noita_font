use std::{
    cmp::{max, min},
    env::args,
    fs,
};

use fontdue::Metrics;
use image::ImageEncoder;

fn map(v: u8, c: u8) -> u8 {
    if v > c {
        255
    } else {
        0
    }
}

// font, size, clip, out
fn main() {
    let arg = args();
    let args_v: Vec<String> = arg.into_iter().skip(1).collect();
    if args_v.len() != 4 {
        panic!("noittf font: str size: f32 clip: u8 out: str\nyou didn't do this and so now we crash\ndo like noittf arial.ttf 32 127 arial");
    }
    let font = &args_v[0];
    let size = args_v[1].parse::<f32>().expect("invalid size");
    let clip = args_v[2].parse::<u8>().expect("invalid clip");
    let out = &args_v[3];
    let font_bytes = fs::read(font).expect("font file doesn't exist!");
    let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();
    let mut pictures: Vec<(Metrics, Vec<u8>)> = Vec::new();
    let mut width = 0;
    let mut height = 0;
    let mut high = 0;
    let mut low = 0;
    for i in 32..=126 {
        let c = i as u8 as char;
        let (metrics, bitmap) = font.rasterize(c, size);
        let mut new = Vec::new();
        for e in bitmap {
            for _ in 0..4 {
                new.push(map(e, clip));
            }
        }
        let ymin = metrics.ymin;
        let mh = metrics.height;
        println!("{c} {height} {ymin} {mh}");
        height = max(height, metrics.height);
        high = max(high, metrics.ymin + metrics.height as i32);
        low = min(low, metrics.ymin);
        width += metrics.width;
        pictures.push((metrics, new));
    }
    println!("{high}");
    high -= height as i32;
    println!("{high}");
    low *= -1;
    width += pictures.len() + 1;
    let mut buf: Vec<u8> = Vec::new();
    let img = image::codecs::png::PngEncoder::new(&mut buf);
    let bh = height as i32 + high + low;
    let mut image_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::new(width as u32, bh as u32);
    let mut xoff = 0;
    let mut char_id = 32;
    let mut xml = format!(
        "
<FontData>	
	<Texture>
		mods/MODNAME/fonts/{out}.png
	</Texture>
	<LineHeight>
		{bh}
	</LineHeight>
	<CharSpace>
		0
	</CharSpace>
	<WordSpace>
		6
	</WordSpace>
"
    )
    .to_owned();
    for picture in pictures {
        let w = picture.0.width;
        let ow = picture.0.advance_width;
        let ideal_height = high - picture.0.ymin - picture.0.height as i32 + height as i32;
        if picture.0.width != 0 && picture.0.height != 0 {
            let picture_img = image::ImageBuffer::from_vec(
                picture.0.width as u32,
                picture.0.height as u32,
                picture.1,
            )
            .unwrap();
            image::imageops::overlay(&mut image_buf, &picture_img, xoff, ideal_height as i64);
        }
        xml.push_str(&format!("\t<QuadChar id=\"{char_id}\" offset_x=\"0\" offset_y=\"0\" rect_h=\"{bh}\" rect_w=\"{w}\" rect_x=\"{xoff}\" rect_y=\"0\" width=\"{ow}\" ></QuadChar>\n"));
        xoff += picture.0.width as i64 + 1;
        char_id += 1;
    }
    xml.push_str("</FontData>\n");
    img.write_image(
        &image_buf,
        width as u32,
        (height as i32 + high + low) as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
    fs::write(format!("{out}.png"), buf).unwrap();
    fs::write(format!("{out}.xml"), xml).unwrap();
}
