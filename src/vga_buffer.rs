use core::fmt;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position   : 0,
        color_code     : ColorCode::new(Color::White, Color::Black),
        buffer         : unsafe { &mut *(0xb8000 as *mut Buffer) },
        mode           : Mode::Normal,
        csi_sequence   : CsiSeq {
            index: 0,
            array: [0; SGR_BUFFER_LENGTH]
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize  = 80;
const SGR_BUFFER_LENGTH: usize = 5;

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15
}

impl Color {
    fn from_code(val: u32) -> Color {
        match val {
            0  => Color::Black,
            1  => Color::Red,
            2  => Color::Green,
            3  => Color::Brown,
            4  => Color::Blue,
            5  => Color::Magenta,
            6  => Color::Cyan,
            7  => Color::LightGray,
            8  => Color::DarkGray,
            9  => Color::LightRed,
            10 => Color::LightGreen,
            11 => Color::Yellow,
            12 => Color::LightBlue,
            13 => Color::Pink,
            14 => Color::LightCyan,
            15 => Color::White,
            _  => Color::Black
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    fn set_fg(&mut self, foreground: Color) {
        self.0 = self.0 & 0xf0 | (foreground as u8);
    }

    fn set_bg(&mut self, background: Color) {
        self.0 = self.0 & 0x0f | ((background as u8) << 4);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

#[derive(Debug, Clone, Copy)]
struct CsiSeq {
    index: usize,
    array: [u32; SGR_BUFFER_LENGTH]
}

enum Mode {
    Normal,
    ESC,
    CSI
}

pub struct Writer {
    column_position: usize,
    row_position   : usize,
    color_code     : ColorCode,
    buffer         : &'static mut Buffer,
    mode           : Mode,
    csi_sequence   : CsiSeq
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20...0x7e | b'\n' | b'\r' | 0x08 | 0x1b => self.write_byte(byte),
                _ => self.write_byte(0xfe)
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match self.mode {
            Mode::ESC => {
                self.parse_esc(byte);
            },
            Mode::CSI => {
                self.parse_csi(byte);
            },
            Mode::Normal => {
                match byte {
                    b'\n' => {
                        self.new_line();
                        self.mode = Mode::Normal
                    },
                    b'\r' => self.column_position = 0,
                    0x1b => {
                        self.mode = Mode::ESC;
                    },
                    0x08 => {  // backspace
                        if self.column_position > 0 {
                            self.column_position -= 1
                        }
                    },
                    byte  => {
                        if self.column_position >= BUFFER_WIDTH {
                            self.new_line();
                        }

                        let row = self.row_position;
                        let col = self.column_position;

                        let color = self.color_code;
                        self.buffer.chars[row][col].write(ScreenChar {
                            ascii_character: byte,
                            color_code     : color
                        });
                        self.column_position += 1;
                    }
                }
            }
        }
    }

    /// Parse ESC sequence
    /// See: https://en.wikipedia.org/wiki/ANSI_escape_code#Escape_sequences
    fn parse_esc(&mut self, byte: u8) {
        match byte {
            b'[' => {
                self.mode = Mode::CSI;
                self.csi_sequence = CsiSeq {
                    index: 0,
                    array: [0; SGR_BUFFER_LENGTH]
                };
            },
            _ => {
                self.mode = Mode::Normal;
                self.write_string("\x1B[37mESC");
                self.write_byte(byte);
                self.write_string("\x1B[0m");
            }
        }
    }

    /// Parse CSI sequence
    /// See: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences
    fn parse_csi(&mut self, byte: u8) {
        match byte {
            b'm' => {   // SGR end
                self.parse_sgr();
                self.mode = Mode::Normal;
            },
            b';' => {
                self.csi_sequence.index += 1;
            },
            b'0'...b'9' => {
                self.mode = Mode::CSI;

                let value = (byte - b'0') as u32;
                let index = self.csi_sequence.index;
                if index < self.csi_sequence.array.len() {
                    self.csi_sequence.array[index] = self.csi_sequence.array[index] * 10 + value;
                }
            },
            _ => {
                use core::fmt::Write;

                self.mode = Mode::Normal;
                self.write_string("\x1B[37mESC[");
                for i in 0..(self.csi_sequence.index + 1) {
                    let value = self.csi_sequence.array[i];
                    write!(self, "{}{}", if i > 0 {";"} else {""}, value).unwrap();
                }
                self.write_byte(byte);
                self.write_string("\x1B[0m");
            }
        }
    }

    /// Parses the stored SGR sequence.
    /// See: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
    fn parse_sgr(&mut self) {
        for i in 0..(self.csi_sequence.index + 1) {
            let value = self.csi_sequence.array[i];
            match value {
                // reset
                0 => self.color_code = ColorCode::new(Color::White, Color::Black),
                // invert
                7 => {
                    let old_fg = self.color_code.0 & 0x0f;
                    let old_bg = self.color_code.0 >> 4;
                    self.color_code.0 = old_fg << 4 | old_bg;
                },
                // fg
                30...37 => {
                    self.color_code.set_fg(Color::from_code(value - 30));
                },
                // default fg
                39 => self.color_code.set_fg(Color::White),
                // bg
                40...47 => {
                    self.color_code.set_bg(Color::from_code(value - 40));
                },
                // default bg
                49 => self.color_code.set_bg(Color::Black),
                // fg - bright
                90...97 => {
                    self.color_code.set_fg(Color::from_code(value - 90 + 8));
                },
                // bg - bright
                100...107 => {
                    self.color_code.set_bg(Color::from_code(value - 100 + 8));
                },
                _ => {}
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position == BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let char = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(char);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code     : self.color_code
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn construct_writer(row: usize, col: usize) -> Writer {
        use std::boxed::Box;

        let buffer = construct_buffer();
        Writer {
            column_position: col,
            row_position   : row,
            color_code     : ColorCode::new(Color::Blue, Color::Magenta),
            buffer         : Box::leak(Box::new(buffer)),
            mode           : Mode::Normal,
            csi_sequence   : CsiSeq {
                index: 0,
                array: [0; SGR_BUFFER_LENGTH]
            }
        }
    }

    fn construct_buffer() -> Buffer {
        use array_init::array_init;

        Buffer {
            chars: array_init(|_| array_init(|_| Volatile::new(empty_char()))),
        }
    }

    fn empty_char() -> ScreenChar {
        ScreenChar {
            ascii_character: b' ',
            color_code     : ColorCode::new(Color::Green, Color::Brown),
        }
    }

    #[test]
    fn write_formatted() {
        use core::fmt::Write;

        let mut writer = construct_writer(BUFFER_HEIGHT - 1, 0);
        writeln!(&mut writer, "a").unwrap();
        writeln!(&mut writer, "b{}", "c").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == BUFFER_HEIGHT - 3 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'a');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'b');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 1 {
                    assert_eq!(screen_char.ascii_character, b'c');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i >= BUFFER_HEIGHT - 2 {
                    assert_eq!(screen_char.ascii_character, b' ');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn write_extra_chars() {
        use core::fmt::Write;

        let mut writer = construct_writer(0, 0);
        write!(&mut writer, "\x08\x08\x08\x08\x08").unwrap();
        write!(&mut writer, "foo").unwrap();
        write!(&mut writer, "\rbar").unwrap();
        write!(&mut writer, "\x08z").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == 0 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'b');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == 0 && j == 1 {
                    assert_eq!(screen_char.ascii_character, b'a');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == 0 && j == 2 {
                    assert_eq!(screen_char.ascii_character, b'z');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn write_unsupported_chars() {
        use core::fmt::Write;

        let mut writer = construct_writer(0, 0);
        writeln!(&mut writer, "漢字").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == 0 && j < 6 {
                    assert_eq!(screen_char.ascii_character, 0xfe);
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn line_feed() {
        use core::fmt::Write;

        let mut writer = construct_writer(0, BUFFER_WIDTH - 1);
        writeln!(&mut writer, "01").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == 0 && j == BUFFER_WIDTH - 1 {
                    assert_eq!(screen_char.ascii_character, b'0');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == 1 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'1');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }

        for row in 0..BUFFER_HEIGHT - 1 {
            writeln!(&mut writer, "{}", row % 10).unwrap();
        }

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i < BUFFER_HEIGHT - 1 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'0' + (i as u8 % 10));
                } else {
                    assert_eq!(screen_char.ascii_character, b' ');
                }
            }
        }
    }

    #[test]
    fn ansi_esc_bgfg() {
        use core::fmt::Write;

        let mut writer = construct_writer(0, 0);
        let empty_color = empty_char().color_code;

        for row in 0..16 {
            let bg_offset = if row < 8 { 40 } else { 100 - 8 };
            for col in 0..16 {
                let fg_offset = if col < 8 { 30 } else { 90 - 8 };
                write!(&mut writer, "\x1B[{};{}m{:X}", bg_offset + row, fg_offset + col, col).unwrap();
            }
            writeln!(&mut writer, "\x1B[0m").unwrap();
        }

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if j < 16 && i < 16 {
                    let expexted_char = if j < 10 { b'0' + j as u8 } else { b'A' + (j - 10) as u8 };
                    assert_eq!(screen_char.ascii_character, expexted_char);
                    let expexted_color = ColorCode::new(
                        Color::from_code(j as u32),
                        Color::from_code(i as u32)
                    );
                    assert_eq!(screen_char.color_code, expexted_color);
                } else {
                    assert_eq!(screen_char.ascii_character, b' ');
                    assert_eq!(screen_char.color_code, empty_color);
                }
            }
        }
    }
}