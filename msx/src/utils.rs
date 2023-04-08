pub fn hexdump(buffer: &[u8], start: u16, end: u16) -> String {
    let mut str = String::new();
    let mut addr = start;
    while addr < end {
        let mut line = format!("{:04x}: ", addr);
        let mut chars = String::new();
        for _ in 0..16 {
            if addr <= end {
                let byte = buffer[addr as usize];
                line.push_str(&format!("{:02x} ", byte));
                let c = byte as char;
                chars.push(if c.is_ascii_graphic() || c == ' ' {
                    c
                } else {
                    '.'
                });

                addr = addr.wrapping_add(1);
            }
        }

        let dump_line = format!("{:>54} {}\n", line, chars);
        str.push_str(&dump_line);

        if addr == 0 {
            break;
        }
    }

    str
}
