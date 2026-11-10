/// Downscales a PNG so its longest side is at most `max_side`, returning PNG bytes.
/// Images already within bounds are returned unchanged.
pub fn make_thumbnail(png_bytes: &[u8], max_side: u32) -> Option<Vec<u8>> {
    let (width, height, rgba) = decode_rgba(png_bytes)?;

    if width.max(height) <= max_side {
        return Some(png_bytes.to_vec());
    }

    let scale = max_side as f64 / width.max(height) as f64;
    let dst_w = ((width as f64 * scale).round() as u32).max(1);
    let dst_h = ((height as f64 * scale).round() as u32).max(1);
    let scaled = box_downscale(&rgba, width, height, dst_w, dst_h);

    encode_rgba(&scaled, dst_w, dst_h)
}

fn decode_rgba(png_bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(png_bytes);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());

    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };

    Some((info.width, info.height, rgba))
}

fn box_downscale(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let (src_w, src_h, dst_w, dst_h) = (src_w as usize, src_h as usize, dst_w as usize, dst_h as usize);
    let mut dst = vec![0u8; dst_w * dst_h * 4];

    for dy in 0..dst_h {
        let y0 = dy * src_h / dst_h;
        let y1 = ((dy + 1) * src_h / dst_h).max(y0 + 1);
        for dx in 0..dst_w {
            let x0 = dx * src_w / dst_w;
            let x1 = ((dx + 1) * src_w / dst_w).max(x0 + 1);

            let mut sum = [0u64; 4];
            for y in y0..y1 {
                let row = y * src_w * 4;
                for x in x0..x1 {
                    let i = row + x * 4;
                    for c in 0..4 {
                        sum[c] += src[i + c] as u64;
                    }
                }
            }

            let count = ((y1 - y0) * (x1 - x0)) as u64;
            let o = (dy * dst_w + dx) * 4;
            for c in 0..4 {
                dst[o + c] = (sum[c] / count) as u8;
            }
        }
    }

    dst
}

fn encode_rgba(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let pixels: Vec<u8> = (0..width * height).flat_map(|_| color).collect();
        encode_rgba(&pixels, width, height).unwrap()
    }

    #[test]
    fn test_large_image_is_scaled_to_max_side_keeping_aspect_ratio() {
        let thumb = make_thumbnail(&solid_png(1000, 500, [10, 20, 30, 255]), 200).unwrap();
        let (w, h, rgba) = decode_rgba(&thumb).unwrap();

        assert_eq!((w, h), (200, 100));
        assert_eq!(&rgba[0..4], &[10, 20, 30, 255]);
    }

    #[test]
    fn test_small_image_is_returned_unchanged() {
        let original = solid_png(50, 40, [1, 2, 3, 4]);
        assert_eq!(make_thumbnail(&original, 200).unwrap(), original);
    }

    #[test]
    fn test_invalid_png_returns_none() {
        assert!(make_thumbnail(&[1, 2, 3, 4, 5], 200).is_none());
    }
}
