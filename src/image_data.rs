use std::io::Cursor;

use image::{ImageFormat, ImageReaderOptions, Limits};

const MAX_IMAGE_WIDTH: u32 = 8192;
const MAX_IMAGE_HEIGHT: u32 = 8192;
const MAX_DECODE_ALLOC: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn decode(bytes: &[u8]) -> Result<DecodedImage, String> {
    let format = image::guess_format(bytes)
        .map_err(|error| format!("unable to identify image format: {error}"))?;

    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP) {
        return Err(format!("image format {format:?} is not enabled yet"));
    }

    let mut limits = Limits::no_limits();
    limits.max_image_width = Some(MAX_IMAGE_WIDTH);
    limits.max_image_height = Some(MAX_IMAGE_HEIGHT);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);

    let mut reader = ImageReaderOptions::new(Cursor::new(bytes));
    reader.set_format(format);
    reader.limits(limits);

    let decoded = reader
        .decode()
        .map_err(|error| format!("image decode failed: {error}"))?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(DecodedImage {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn rejects_unknown_binary_data() {
        assert!(decode(b"this is not an image").is_err());
    }
}
