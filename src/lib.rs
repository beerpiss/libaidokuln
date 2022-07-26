// https://github.com/pandeynmn/nmns-extensions/blob/novel-branch/src/LNInterceptor.ts
// Originally made by Jim, ported by beerpsi
#![no_std]
#![feature(test)]
#![allow(clippy::needless_range_loop)]
#[cfg(test)]
mod tests;

#[cfg(test)]
mod bench;

pub mod fonts;
use fonts::Font;

extern crate alloc;
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};

const BMP_HEADER1: [u8; 2] = [0x42, 0x4D];
const BMP_HEADER2: [u8; 12] = [
    0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00,
];
const BMP_HEADER3: [u8; 8] = [0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00];
const BMP_HEADER4: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// horizontal, vertical
#[derive(Debug)]
pub struct Padding(pub f32, pub f32);

#[derive(Debug)]
pub struct ImageOptions {
    pub text_color: usize,
    pub background_color: usize,
    pub padding: Padding,
    pub width: f32,
    pub constant_width: bool,
}

#[derive(Debug, PartialEq)]
pub struct Spliterated {
    pub split: Vec<String>,
    pub width: f32,
}

fn ceil(num: f32) -> f32 {
    (num as i32 + 1) as f32
}

fn calculate_text_length<T: AsRef<str>>(text: T, font: &Font) -> f32 {
    let mut ret = 0.0;
    for c in text.as_ref().as_bytes() {
        let curr = *c;
        let idx = if (b' '..=127).contains(&curr) {
            (curr - 32) as usize
        } else {
            0
        };
        ret += (font.font[idx].len() as f32) / font.height;
    }
    ret
}

pub fn break_apart<T: AsRef<str>>(text: T, max_width: f32, font: &Font) -> Spliterated {
    let width = calculate_text_length(&text, font);
    if width <= max_width {
        return Spliterated {
            split: vec![String::from(text.as_ref())],
            width,
        };
    }

    let text = text.as_ref().replace('\n', "\n ");
    let fullsplit = text
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mut split = Vec::new();

    let mut base = 0;
    let mut maxlen = 0.0;
    #[allow(unused_assignments)]
    let mut prevlen = 0.0;
    let mut curlen = 0.0;

    for i in 0..fullsplit.len() {
        prevlen = curlen;
        curlen = calculate_text_length(fullsplit[base..i + 1].join(" "), font);
        if curlen > max_width || (i >= 1 && fullsplit[i - 1].contains('\n')) {
            split.push(fullsplit[base..i].join(" ").replace('\n', ""));
            if prevlen > maxlen {
                maxlen = prevlen;
            }
            base = i;
        }
    }

    split.push(fullsplit[base..].join(" "));
    if curlen > maxlen {
        maxlen = curlen;
    }
    Spliterated {
        split,
        width: maxlen,
    }
}

fn split_color(color: usize) -> Vec<u8> {
    vec![
        (color & 0xFF) as u8,
        ((color & 0xFF00) >> 8) as u8,
        ((color & 0xFF0000) >> 16) as u8,
    ]
}

/// Turns text into a 3-dimensional array containing the color data for each pixel
pub fn write_text<T: AsRef<str>>(text: T, font: Font, options: ImageOptions) -> Vec<Vec<Vec<u8>>> {
    let text = text.as_ref();
    // let text = text.as_ref().replace(|c| matches!(c, '\0'..='\x7F'), "");
    let spliterated = break_apart(text, options.width - options.padding.0 * 2.0, &font);
    let split = spliterated.split;
    let width = if options.constant_width {
        options.width
    } else {
        spliterated.width + 2.0 * options.padding.0
    };
    let height = (split.len() as f32) * font.height + options.padding.1 * 2.0;

    let bg = split_color(options.background_color);
    let mut img = vec![vec![bg; ceil(width) as usize]; ceil(height) as usize];
    let mut line_at: usize = 0;

    for i in 0..(ceil(height) as usize) {
        if (i as f32) < options.padding.1 || (i as f32) >= height - options.padding.1 {
            continue;
        }

        if (i as f32 - options.padding.1) % font.height == 0.0 {
            line_at += 1;
        }

        let mut letter_on: usize = 0;
        let mut letter = Vec::new();
        let mut letter_base = options.padding.0;
        let bytes = split[line_at - 1].as_bytes();
        for j in 0..(ceil(width) as usize) {
            if (j as f32) < options.padding.0 || (j as f32) >= width - options.padding.0 {
                continue;
            }

            if (j as f32) >= letter_base + (letter.len() as f32) / font.height {
                letter_on += 1;
                if letter_on > bytes.len() {
                    continue;
                }
                letter_base = j as f32;
                let mut char = bytes[letter_on - 1].saturating_sub(32);
                if char >= 95 {
                    char = 0;
                }
                letter = font.font[char as usize].to_owned();
            }

            let thing = ((i as f32 - options.padding.1) - ((line_at - 1) as f32) * font.height)
                * ((letter.len() as f32) / font.height)
                + (j as f32 - letter_base);
            let alpha = letter[thing as usize];

            if alpha != 0 {
                let colors = split_color(options.text_color);
                img[i][j] = vec![
                    core::cmp::min(255, colors[0] * alpha / 255 + colors[0] * (1 - alpha / 255)),
                    core::cmp::min(255, colors[1] * alpha / 255 + colors[1] * (1 - alpha / 255)),
                    core::cmp::min(255, colors[2] * alpha / 255 + colors[2] * (1 - alpha / 255)),
                ];
            }
        }
    }

    img
}

fn little_endian(size: usize, data: usize) -> Vec<u8> {
    let mut ret = Vec::new();
    for i in 0..size {
        ret.push((data >> (8 * i) & 0x000000ff) as u8);
    }
    ret
}

pub fn write_image_data(data: &mut Vec<Vec<Vec<u8>>>) -> Vec<u8> {
    let mut imgdata: Vec<u8> = Vec::new();
    let width = data[0].len();
    let bytewidth = (((width as f32) * 3.0 / 4.0) + 0.5) as usize * 4;
    let height = data.len();
    let size = bytewidth * height;
    let file_size = size + 54;
    for i in (0..height).rev() {
        for j in 0..width {
            imgdata.append(&mut data[i][j]);
        }
        imgdata.append(&mut vec![0; bytewidth - width * 3]);
    }
    let mut ret = Vec::with_capacity(file_size);
    ret.append(&mut BMP_HEADER1.to_vec());
    ret.append(&mut little_endian(4, file_size));
    ret.append(&mut BMP_HEADER2.to_vec());
    ret.append(&mut little_endian(4, width));
    ret.append(&mut little_endian(4, height));
    ret.append(&mut BMP_HEADER3.to_vec());
    ret.append(&mut little_endian(4, size));
    ret.append(&mut vec![0x13, 0x0b, 0x00, 0x00, 0x13, 0x0b, 0x00, 0x00]);
    ret.append(&mut BMP_HEADER4.to_vec());
    ret.append(&mut imgdata);
    ret
}
