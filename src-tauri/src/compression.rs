use image::{GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::Path;

/// Compress an image: resize if larger than max_dimension (preserving aspect ratio), output as PNG.
pub fn compress_image(data: &[u8], max_dimension: u32, _quality: u8) -> Result<Vec<u8>, String> {
    let img =
        image::load_from_memory(data).map_err(|e| format!("Failed to load image: {}", e))?;

    let (w, h) = img.dimensions();

    let img = if w > max_dimension || h > max_dimension {
        // Resize preserving aspect ratio — fit within max_dimension box
        img.resize(max_dimension, max_dimension, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(buf)
}

/// Save original and compressed images to a session directory.
/// Returns (original_path, compressed_path).
pub fn save_capture(
    session_dir: &Path,
    original_data: &[u8],
    max_dimension: u32,
    quality: u8,
) -> Result<(String, String), String> {
    let original_path = session_dir.join("original.png");
    std::fs::write(&original_path, original_data)
        .map_err(|e| format!("Failed to write original: {}", e))?;

    let compressed_data = compress_image(original_data, max_dimension, quality)?;
    let compressed_path = session_dir.join("compressed.png");
    std::fs::write(&compressed_path, &compressed_data)
        .map_err(|e| format!("Failed to write compressed: {}", e))?;

    Ok((
        original_path.to_string_lossy().to_string(),
        compressed_path.to_string_lossy().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
        });
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        buf
    }

    #[test]
    fn test_compress_resizes_large_image() {
        let data = create_test_image(3840, 2160);
        let result = compress_image(&data, 1920, 85).unwrap();
        let img = image::load_from_memory(&result).unwrap();
        assert!(img.width() <= 1920);
        assert!(img.height() <= 1920);
    }

    #[test]
    fn test_compress_preserves_small_image() {
        let data = create_test_image(800, 600);
        let result = compress_image(&data, 1920, 85).unwrap();
        let img = image::load_from_memory(&result).unwrap();
        assert_eq!(img.width(), 800);
        assert_eq!(img.height(), 600);
    }

    #[test]
    fn test_compress_preserves_aspect_ratio() {
        let data = create_test_image(3840, 2160);
        let result = compress_image(&data, 1920, 85).unwrap();
        let img = image::load_from_memory(&result).unwrap();
        let ratio = img.width() as f64 / img.height() as f64;
        let expected_ratio = 3840.0 / 2160.0;
        assert!(
            (ratio - expected_ratio).abs() < 0.02,
            "Aspect ratio mismatch: got {} expected {}",
            ratio,
            expected_ratio
        );
    }

    #[test]
    fn test_save_capture_creates_both_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let data = create_test_image(800, 600);
        let (orig, comp) = save_capture(tmp.path(), &data, 1920, 85).unwrap();
        assert!(std::path::Path::new(&orig).exists());
        assert!(std::path::Path::new(&comp).exists());
    }
}
