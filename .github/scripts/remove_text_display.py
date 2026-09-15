from pathlib import Path

path = Path('src/ui/classic_display.rs')
text = path.read_text()

start = text.index('pub(crate) fn paint(painter: &Painter, display_rect: Rect, value: &str) {')
end = text.index('fn segment_mask(ch: char) -> u8 {', start)
text = text[:start] + text[end:]

start = text.index('fn segment_mask(ch: char) -> u8 {')
end = text.index('fn draw_digit(', start)
text = text[:start] + text[end:]

start = text.index('fn draw_digit(')
end = text.index('fn draw_segment_mask(', start)
text = text[:start] + text[end:]

start = text.index('fn display_cells(')
end = text.index('#[cfg(test)]', start)
text = text[:start] + text[end:]

start = text.index('#[cfg(test)]')
text = text[:start] + r'''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_dimensions_follow_original_classic_display() {
        assert!((CHARACTER_PITCH / REFERENCE_UNITS_PER_MM - 3.81).abs() < 0.0001);
        assert!((CHARACTER_HEIGHT / REFERENCE_UNITS_PER_MM - 2.794).abs() < 0.0001);
        assert!((CHARACTER_WIDTH / REFERENCE_UNITS_PER_MM - 1.5748).abs() < 0.0001);
        assert!((DECIMAL_DIAMETER / REFERENCE_UNITS_PER_MM - 0.5334).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH / REFERENCE_UNITS_PER_MM - 57.15).abs() < 0.001);
        assert_eq!(CHARACTER_COUNT, 15);
        assert_eq!(MODULE_COUNT * CHARACTERS_PER_MODULE, CHARACTER_COUNT);
    }

    #[test]
    fn photographed_glass_keeps_original_led_to_aperture_ratio() {
        let pitch_ratio = CHARACTER_PITCH / REFERENCE_DISPLAY_WIDTH;
        let height_ratio = CHARACTER_HEIGHT / REFERENCE_DISPLAY_HEIGHT;
        assert!((pitch_ratio - 0.05444).abs() < 0.0001);
        assert!((height_ratio - 0.19749).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH / REFERENCE_DISPLAY_WIDTH - 0.8165).abs() < 0.0002);
    }

    #[test]
    fn raw_hardware_segment_bits_match_renderer_wiring() {
        assert_eq!(SEG_A, 0x01);
        assert_eq!(SEG_G, 0x40);
        assert_eq!(SEG_DP, 0x80);
        assert_eq!(SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F, 0x3f);
    }
}
'''
path.write_text(text)

doc = Path('docs/files/src/ui/classic_display.rs.md')
doc.write_text(doc.read_text().replace(
    'The older text renderer remains only as isolated transitional/test support and is no longer on the panel display path.',
    'The former text-to-cell renderer has been removed from this module, leaving raw segment masks as the only display input path.'
))
