//! Vector lettering and reconstructed math. Provenance: tools/typography/README.md.
use std::{collections::HashMap, sync::OnceLock};
pub struct Glyph {
    pub advance: f32,
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub contours: Vec<Vec<[f32; 2]>>,
}
pub fn glyph(c: char, weight: u16) -> &'static Glyph {
    static GLYPHS: OnceLock<HashMap<(u16, char), Glyph>> = OnceLock::new();
    GLYPHS
        .get_or_init(|| {
            let mut bytes = include_bytes!("lettering.bin").as_slice();
            fn read<const N: usize>(bytes: &mut &[u8]) -> [u8; N] {
                let (head, tail) = bytes.split_at(N);
                *bytes = tail;
                head.try_into().unwrap()
            }
            let mut map = HashMap::new();
            while !bytes.is_empty() {
                let weight = u16::from_le_bytes(read(&mut bytes));
                let c = char::from_u32(u32::from_le_bytes(read(&mut bytes))).unwrap();
                let advance = f32::from_le_bytes(read(&mut bytes));
                let nv = u32::from_le_bytes(read(&mut bytes));
                let ni = u32::from_le_bytes(read(&mut bytes));
                let vertices = (0..nv)
                    .map(|_| {
                        [
                            f32::from_le_bytes(read(&mut bytes)),
                            f32::from_le_bytes(read(&mut bytes)),
                        ]
                    })
                    .collect();
                let indices = (0..ni)
                    .map(|_| u16::from_le_bytes(read(&mut bytes)) as u32)
                    .collect();
                let nc = u16::from_le_bytes(read(&mut bytes));
                let contours = (0..nc)
                    .map(|_| {
                        let count = u16::from_le_bytes(read(&mut bytes));
                        (0..count)
                            .map(|_| {
                                [
                                    f32::from_le_bytes(read(&mut bytes)),
                                    f32::from_le_bytes(read(&mut bytes)),
                                ]
                            })
                            .collect()
                    })
                    .collect();
                map.insert(
                    (weight, c),
                    Glyph {
                        advance,
                        vertices,
                        indices,
                        contours,
                    },
                );
            }
            map
        })
        .get(&(weight, c))
        .expect("missing HP-67 outline glyph")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_math_contours_keep_the_crossing_filled_and_the_y_descender() {
        for weight in [400, 600, 700] {
            let x = glyph('x', weight);
            let point = [0.375, 0.605];
            let cross = |a: [f32; 2], b: [f32; 2]| a[0] * b[1] - a[1] * b[0];
            assert!(x.indices.chunks_exact(3).any(|tri| {
                let v: Vec<_> = tri.iter().map(|i| x.vertices[*i as usize]).collect();
                let signs: Vec<_> = (0..3)
                    .map(|i| {
                        let a = v[i];
                        let b = v[(i + 1) % 3];
                        cross(
                            [b[0] - a[0], b[1] - a[1]],
                            [point[0] - a[0], point[1] - a[1]],
                        )
                    })
                    .collect();
                signs.iter().all(|s| *s >= -0.00001) || signs.iter().all(|s| *s <= 0.00001)
            }));
            assert!(glyph('y', weight).vertices.iter().any(|v| v[1] > 1.2));
            assert_eq!(glyph('\u{03c0}', weight).contours.len(), 3);
        }
    }
}
