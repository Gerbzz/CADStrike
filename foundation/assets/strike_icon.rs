//! Original local Strike candidate icons. Geometry and pixels are generated in Rust;
//! no PhotoCraft, CADCraft or ArtCraft mark is embedded in the candidate executables.

#[derive(Clone, Copy)]
pub enum Kind {
    Cad,
    Skin,
}

pub fn rgba(size: u32, kind: Kind) -> Vec<u8> {
    let mut out = Vec::with_capacity(size as usize * size as usize * 4);
    for y in 0..size {
        for x in 0..size {
            let mut coverage = 0.0_f32;
            let mut rgb = [0.0_f32; 3];
            for yy in 0..4 {
                for xx in 0..4 {
                    let u = (x as f32 + (xx as f32 + 0.5) / 4.0) / size as f32;
                    let v = (y as f32 + (yy as f32 + 0.5) / 4.0) / size as f32;
                    let dx = (u - 0.5).abs() - 0.35;
                    let dy = (v - 0.5).abs() - 0.35;
                    if dx.max(0.0).hypot(dy.max(0.0)) > 0.13 {
                        continue;
                    }
                    coverage += 1.0;
                    let glyph = match kind {
                        Kind::Cad => (0.27..=0.73).contains(&u) && (0.24..=0.76).contains(&v) && (u <= 0.39 || v <= 0.36 || v >= 0.64),
                        Kind::Skin => [(0.24, 0.36), (0.44, 0.56), (0.64, 0.76)]
                            .iter()
                            .any(|&(top, bottom)| (top..=bottom).contains(&v) && (0.25 + (v - top) * 0.36..=0.73 + (v - top) * 0.36).contains(&u)),
                    };
                    let c = if glyph {
                        match kind {
                            Kind::Cad => [74.0, 220.0, 211.0],
                            Kind::Skin => [210.0, 154.0, 248.0],
                        }
                    } else {
                        [18.0 + 14.0 * v, 27.0 + 10.0 * v, 45.0 + 12.0 * v]
                    };
                    for i in 0..3 {
                        rgb[i] += c[i];
                    }
                }
            }
            let alpha = coverage / 16.0;
            for c in rgb {
                out.push(if coverage == 0.0 { 0 } else { (c / coverage).round() as u8 });
            }
            out.push((alpha * 255.0).round() as u8);
        }
    }
    out
}

/// Write four uncompressed ARGB bitmap entries in the standard Windows ICO container.
/// The build script embeds this generated icon; the same pixels supply the runtime window icon.
pub fn ico(kind: Kind) -> Vec<u8> {
    let sizes = [16_u32, 32, 64, 256];
    let mut images = Vec::new();
    for size in sizes {
        let pixels = rgba(size, kind);
        let mask_stride = size.div_ceil(32) as usize * 4;
        let mut data = Vec::with_capacity(40 + size as usize * size as usize * 4 + mask_stride * size as usize);
        data.extend_from_slice(&40_u32.to_le_bytes());
        data.extend_from_slice(&(size as i32).to_le_bytes());
        data.extend_from_slice(&((size * 2) as i32).to_le_bytes());
        data.extend_from_slice(&1_u16.to_le_bytes());
        data.extend_from_slice(&32_u16.to_le_bytes());
        data.extend_from_slice(&0_u32.to_le_bytes());
        data.extend_from_slice(&(size * size * 4).to_le_bytes());
        data.extend_from_slice(&[0; 16]);
        for y in (0..size as usize).rev() {
            for x in 0..size as usize {
                let p = &pixels[(y * size as usize + x) * 4..][..4];
                data.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
            }
        }
        for y in (0..size as usize).rev() {
            let mut row = vec![0_u8; mask_stride];
            for x in 0..size as usize {
                if pixels[(y * size as usize + x) * 4 + 3] == 0 {
                    row[x / 8] |= 0x80 >> (x % 8);
                }
            }
            data.extend_from_slice(&row);
        }
        images.push(data);
    }
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + sizes.len() * 16;
    for (size, image) in sizes.into_iter().zip(&images) {
        out.extend_from_slice(&[size as u8, size as u8, 0, 0]);
        out.extend_from_slice(&1_u16.to_le_bytes());
        out.extend_from_slice(&32_u16.to_le_bytes());
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        out.extend_from_slice(&image);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_marks_are_distinct_and_icons_have_four_sizes() {
        assert_ne!(rgba(64, Kind::Cad), rgba(64, Kind::Skin));
        for kind in [Kind::Cad, Kind::Skin] {
            let mark = rgba(64, kind);
            assert_eq!(mark.len(), 64 * 64 * 4);
            assert_eq!(mark[3], 0);
            assert!(mark.as_chunks::<4>().0.iter().any(|p| p[3] == 255));
            let icon = ico(kind);
            assert_eq!(&icon[..6], &[0, 0, 1, 0, 4, 0]);
        }
    }
}
