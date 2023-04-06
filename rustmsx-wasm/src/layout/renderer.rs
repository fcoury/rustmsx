use msx::TMS9918;

pub struct Renderer<'a> {
    vdp: &'a TMS9918,
    pub screen_buffer: [u8; 256 * 192],
}

impl<'a> Renderer<'a> {
    pub fn new(vdp: &'a TMS9918) -> Self {
        let screen_buffer = [0; 256 * 192];
        Self { vdp, screen_buffer }
    }

    pub fn draw(&mut self, _x0: u16, y0: u16, _x1: u16, y1: u16) {
        // TODO check for text mode
        // TODO check for scroll delta
        let fg = 15; // TODO Pixel fg = palFg[vdp.getForegroundColor()];
        let bg = 4; // TODO Pixel bg = palBg[vdp.getBackgroundColor()];

        let screen_mode = 0;
        let height = y1 - y0;

        for y in y0..height {
            // renders this raster line
            match screen_mode {
                0 => {
                    self.render_text1(y as usize, fg, bg);
                }
                _ => panic!("Unsupported screen mode: {}", screen_mode),
            }
        }
    }

    pub fn render_text1(&mut self, line: usize, fg: u8, bg: u8) {
        let pattern_area = self.vdp.pattern_table();
        let l = (line + self.vdp.get_vertical_scroll()) & 7;

        // Calculate the base address of the PNT using register R#2
        let pnt_base = (self.vdp.registers[2] as usize & 0x0F) * 0x0400;

        let name_start = (line / 8) * 40;
        let name_end = name_start + 40;
        let mut pixel_ptr = line * 256;
        for name in name_start..name_end {
            // FIXME why is the screen content at 0x0990 in our version?
            let screen_offset = pnt_base + name; // Calculate the proper offset in the VRAM
            let char_code = self.vdp.vram[screen_offset]; // Get the value directly from the VRAM array
            let pattern = pattern_area[l + char_code as usize * 8];

            for i in 0..6 {
                let mask = 0x80 >> i;
                self.screen_buffer[pixel_ptr + i] = if (pattern & mask) != 0 { fg } else { bg };
            }

            pixel_ptr += 6;
        }
    }
}
