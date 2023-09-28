use std::collections::HashMap;
use crate::{Buffer, BitFont, Glyph, TextPane, TextAttribute};

enum GlyphShape {
    Whitespace,
    Block,
    Mixed
}

/// Reduces the amount of color changes inside a buffer.
/// Ignoring foreground color changes on whitespaces and background color changes on blocks.
/// 
/// That reduces the amount of color switches required in the output formats.
pub(crate) struct ColorOptimizer {
    shape_map: HashMap<usize, HashMap<char, GlyphShape>>,
}

impl ColorOptimizer {
    pub fn new(buf: &Buffer) -> Self {
        let shape_map = generate_shape_map(buf);
        Self {
            shape_map,
        }
    }

    pub fn optimize(&self, buffer: &Buffer) -> Buffer {
        let mut b = buffer.flat_clone();
        for layer in &mut b.layers {
            let mut cur_attr = TextAttribute::default();
            for y in 0..layer.get_height() {
                for x in 0..layer.get_width() {
                    let ch = layer.get_char((x, y));
                    let map  = self.shape_map.get(&ch.get_font_page()).unwrap();
                    let mut attribute = ch.attribute;
                    match *map.get(&ch.ch).unwrap() {
                        GlyphShape::Whitespace => {
                            attribute.set_foreground(cur_attr.get_foreground());
                        },
                        GlyphShape::Block => {
                            attribute.set_background(cur_attr.get_background());
                        },
                        GlyphShape::Mixed => {
                        },
                    }
                    layer.set_char((x, y), crate::AttributedChar { ch: ch.ch, attribute });
                    cur_attr = attribute;
                }
            }
        }
        b
    }
}

fn generate_shape_map(buf: &Buffer) -> HashMap<usize, HashMap<char, GlyphShape>> {
    let mut shape_map = HashMap::new();
    for (slot, font) in buf.font_iter() {
        let mut font_map = HashMap::new();
        for (char, glyph) in &font.glyphs {
            font_map.insert(*char, get_shape(font, glyph));
        }
        shape_map.insert(*slot, font_map);
    }
    shape_map
}

fn get_shape(font: &BitFont, glyph: &Glyph) -> GlyphShape {
    let mut ones = 0;
    for row in &glyph.data {
        ones += row.count_ones();
    }
    if ones == 0 {
        GlyphShape::Whitespace
    } else if ones == font.size.width as u32 * font.size.height as u32 {
        GlyphShape::Block
    } else {
        GlyphShape::Mixed
    }
}