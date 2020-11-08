use image::png::PngDecoder;
use image::ImageDecoder;

use std::io;


pub fn load_image(buffer: &[u8]) -> TextureImage2D {
    let cursor = io::Cursor::new(buffer);
    let image_decoder = PngDecoder::new(cursor).unwrap();
    let (width, height) = image_decoder.dimensions();
    let total_bytes = image_decoder.total_bytes();
    let bytes_per_pixel = image_decoder.color_type().bytes_per_pixel() as u32;
    let mut image_data = vec![0 as u8; total_bytes as usize];
    image_decoder.read_image(&mut image_data).unwrap();

    assert_eq!(total_bytes, (width * height * bytes_per_pixel) as u64);

    TextureImage2D::new(width, height, bytes_per_pixel, image_data)
}


#[derive(Clone)]
pub struct TextureImage2D {
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
    data: Vec<u8>,
}

impl TextureImage2D {
    pub fn new(width: u32, height: u32, bytes_per_pixel: u32, data: Vec<u8>) -> Self {
        Self {
            width: width,
            height: height,
            bytes_per_pixel: bytes_per_pixel,
            data: data,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

