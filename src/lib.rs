use core::fmt;

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView, ImageFormat};
use reqwest::header::CONTENT_TYPE;

pub enum ProgramError {
    InvalidInputPath,
    FailedToDecodeInput,
    FailedToWriteToOutput,
    FailedToDownload,
    DownloadInvalid,
}

#[derive(Debug)]
pub struct ProgramParameters<'a> {
    pub input_path: &'a str,
    pub output_path: Option<&'a str>,
    pub output_width: Option<u32>,
    pub symbol_aspect_ratio: f32,
    pub charset: &'a str,
}

enum ImageFormatRes {
    Invalid,
    None,
    Some(ImageFormat),
}

struct AsciiImage {
    dimensions: (u32, u32),
    data: Vec<Vec<char>>,
}

impl fmt::Display for AsciiImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.data {
            for c in line {
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl AsciiImage {
    pub fn create_empty(dimensions: (u32, u32)) -> Self {
        Self {
            dimensions,
            data: vec![vec!['.'; dimensions.0 as usize]; dimensions.1 as usize],
        }
    }

    pub fn create_from(img: &DynamicImage, charset: &str) -> Self {
        let mut ascii_img = Self::create_empty(img.dimensions());
        ascii_img.copy_from(img, charset);
        ascii_img
    }

    pub fn copy_from(&mut self, img: &DynamicImage, charset: &str) {
        assert!(img.dimensions() == self.dimensions);
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let pixel = img.get_pixel(x, y);
                let brightness =
                    (pixel[0] as f32 / u8::MAX as f32) * (pixel[3] as f32 / u8::MAX as f32);
                let num_chars = charset.chars().count();
                let symbol = charset
                    .chars()
                    .nth(brightness_to_index(brightness, num_chars))
                    .unwrap();
                self.data[y as usize][x as usize] = symbol;
            }
        }
    }
}

pub fn generate_image(params: &ProgramParameters) -> Result<(), ProgramError> {
    // let pp = PathBuf::from("http://seznam.cz/image.png");
    // image::ImageFormat::from_mime_type(mime_type)
    // println!("{:?}", pp.extension());
    // image::load_from_memory_with_format(&[0u8;1], image::ImageFormat::from_extension(ext));
    let img = load_image(params.input_path)?;

    let (orig_w, orig_h) = img.dimensions();
    let aspect_ratio = orig_w as f32 / orig_h as f32;

    let w = params.output_width.unwrap_or(orig_w);
    let ascii_art_height = (w as f32 * params.symbol_aspect_ratio / aspect_ratio) as u32;

    let img2 =
        img.grayscale()
            .resize_exact(w, ascii_art_height, image::imageops::FilterType::CatmullRom);

    let ascii_image = AsciiImage::create_from(&img2, params.charset);
    if let Some(output_path) = params.output_path {
        std::fs::write(output_path, ascii_image.to_string())
            .map_err(|_| ProgramError::FailedToWriteToOutput)?;
    } else {
        println!("{ascii_image}");
    }

    Ok(())
}

fn brightness_to_index(brightness: f32, num_chars: usize) -> usize {
    (brightness * num_chars as f32 - 0.5)
        .round()
        .clamp(0.0, num_chars as f32 - 1.0) as usize
}

fn load_image(path: &str) -> Result<DynamicImage, ProgramError> {
    if path.starts_with("http://") || path.starts_with("https://") {
        load_image_from_url(path)
    } else {
        load_image_from_file(path)
    }
}

fn load_image_from_url(path: &str) -> Result<DynamicImage, ProgramError> {
    let x = reqwest::blocking::get(path).map_err(|_| ProgramError::FailedToDownload)?;
    match get_image_format_from_response(&x) {
        ImageFormatRes::Invalid => Err(ProgramError::DownloadInvalid),
        ImageFormatRes::None => {
            let bytes = x.bytes().map_err(|_| ProgramError::DownloadInvalid)?;
            image::load_from_memory(&bytes).map_err(|_| ProgramError::DownloadInvalid)
        }
        ImageFormatRes::Some(format) => {
            let bytes = x.bytes().map_err(|_| ProgramError::DownloadInvalid)?;
            image::load_from_memory_with_format(&bytes, format)
                .map_err(|_| ProgramError::DownloadInvalid)
        }
    }
}

fn load_image_from_file(path: &str) -> Result<DynamicImage, ProgramError> {
    let reader = ImageReader::open(path).map_err(|_| ProgramError::InvalidInputPath)?;
    reader
        .decode()
        .map_err(|_| ProgramError::FailedToDecodeInput)
}

fn get_image_format_from_response(response: &reqwest::blocking::Response) -> ImageFormatRes {
    let headers = response.headers();
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        if let Ok(content_type) = content_type.to_str() {
            if let Some(format) = image::ImageFormat::from_mime_type(content_type) {
                ImageFormatRes::Some(format)
            } else {
                ImageFormatRes::Invalid
            }
        } else {
            ImageFormatRes::None
        }
    } else {
        ImageFormatRes::None
    }
}
