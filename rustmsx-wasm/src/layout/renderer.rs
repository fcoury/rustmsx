use msx::{vdp::DisplayMode, TMS9918};

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

        let height = y1 - y0;

        tracing::trace!("Rendering mode: {:?}", self.vdp.display_mode);

        for y in y0..height {
            // renders this raster line
            match self.vdp.display_mode {
                DisplayMode::Text1 => {
                    self.render_text1(y as usize, fg, bg);
                }
                DisplayMode::Multicolor => {
                    self.render_text2(y as usize, fg, bg);
                }
                DisplayMode::Graphic1 => {
                    self.render_graphic1(y as usize);
                }
                _ => panic!("Unsupported screen mode: {:?}", self.vdp.display_mode),
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

    pub fn render_text2(&mut self, line: usize, fg: u8, bg: u8) {
        let pattern_area = self.vdp.pattern_table();
        let l = (line + self.vdp.get_vertical_scroll()) & 7;

        // Calculate the base address of the PNT using register R#2
        let pnt_base = (self.vdp.registers[2] as usize & 0x0F) * 0x0400;

        let name_start = (line / 8) * 32;
        let name_end = name_start + 32;
        let mut pixel_ptr = line * 256;
        for name in name_start..name_end {
            // FIXME why is the screen content at 0x0990 in our version?
            let screen_offset = pnt_base + name; // Calculate the proper offset in the VRAM
            let char_code = self.vdp.vram[screen_offset]; // Get the value directly from the VRAM array
            let pattern = pattern_area[l + char_code as usize * 8];

            for i in 0..8 {
                let mask = 0x80 >> i;
                self.screen_buffer[pixel_ptr + i] = if (pattern & mask) != 0 { fg } else { bg };
            }

            pixel_ptr += 8;
        }
    }

    pub fn render_graphic1(&mut self, line: usize) {
        let pattern_area = self.vdp.pattern_table();
        let l = line & 7;
        let color_area = self.vdp.color_table();

        let mut scroll = self.vdp.get_horizontal_scroll_high();
        let mut name_ptr = self.get_name_ptr(line, scroll);
        let pixel_ptr = line * 256;
        for _ in 0..32 {
            let char_code = name_ptr[scroll & 0x1F];
            let pattern = pattern_area[l + char_code as usize * 8];
            let color = color_area[char_code as usize / 8];
            let fg = color >> 4;
            let bg = color & 0x0F;
            for i in 0..8 {
                let mask = 0x80 >> i;
                self.screen_buffer[pixel_ptr + i] = if (pattern & mask) != 0 { fg } else { bg };
            }

            scroll += 1;
            if (scroll & 0x1F) == 0 {
                name_ptr = self.get_name_ptr(line, scroll);
            }
        }
    }

    fn get_name_ptr(&self, line: usize, scroll: usize) -> Vec<u8> {
        let base = (self.vdp.registers[2] as usize & 0x0F) * 0x0400;
        let offset = (((line + self.vdp.get_vertical_scroll()) / 8) * 32 + scroll) % 1024;
        self.vdp.vram[base + offset..].to_vec()
    }
}
