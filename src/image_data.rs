use std::io::Cursor;

use image::{ImageFormat, ImageReader, Limits};

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

    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return Err(format!("image format {format:?} is not enabled yet"));
    }

    let mut limits = Limits::no_limits();
    limits.max_image_width = Some(MAX_IMAGE_WIDTH);
    limits.max_image_height = Some(MAX_IMAGE_HEIGHT);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
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

    const TWO_BY_ONE_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
        0x00, 0xf4, 0x22, 0x7f, 0x8a, 0x00, 0x00, 0x00, 0x11, 0x49, 0x44, 0x41, 0x54, 0x78,
        0x9c, 0x63, 0xe4, 0x12, 0x91, 0xfb, 0xcf, 0xc0, 0xc0, 0xc0, 0x00, 0x00, 0x06, 0x9d,
        0x01, 0x3d, 0x34, 0x71, 0x25, 0xeb, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44,
        0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn decodes_real_png_pixels() {
        let decoded = decode(TWO_BY_ONE_PNG).expect("embedded PNG should decode");
        assert_eq!((decoded.width, decoded.height), (2, 1));
        assert_eq!(decoded.rgba.len(), 8);
        assert_eq!(&decoded.rgba[..4], &[10, 20, 30, 255]);
    }

    #[test]
    fn rejects_unknown_binary_data() {
        assert!(decode(b"this is not an image").is_err());
    }
}
