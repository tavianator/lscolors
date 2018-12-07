use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Fixed(u8),
    RGB(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontStyle {
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle {
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl FontStyle {
    pub fn bold() -> Self {
        FontStyle {
            bold: true,
            italic: false,
            underline: false,
        }
    }

    pub fn italic() -> Self {
        FontStyle {
            bold: false,
            italic: true,
            underline: false,
        }
    }

    pub fn underline() -> Self {
        FontStyle {
            bold: false,
            italic: false,
            underline: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub font_style: FontStyle,
}

impl Style {
    /// Parse ANSI escape sequences like `38;2;255;0;100;1;4` (pink, bold, underlined).
    pub fn from_ansi_sequence(code: &str) -> Option<Style> {
        let mut parts: VecDeque<u8> = code
            .split(';')
            .map(|c| u8::from_str_radix(c, 10).ok())
            .collect::<Option<_>>()?;

        let mut font_style = FontStyle::default();
        let mut foreground = None;
        let mut background = None;

        loop {
            match parts.pop_front() {
                Some(0) => font_style = FontStyle::default(),
                Some(1) => font_style.bold = true,
                Some(3) => font_style.italic = true,
                Some(4) => font_style.underline = true,
                Some(30) => foreground = Some(Color::Black),
                Some(31) => foreground = Some(Color::Red),
                Some(32) => foreground = Some(Color::Green),
                Some(33) => foreground = Some(Color::Yellow),
                Some(34) => foreground = Some(Color::Blue),
                Some(35) => foreground = Some(Color::Purple),
                Some(36) => foreground = Some(Color::Cyan),
                Some(37) => foreground = Some(Color::White),
                Some(38) => match (parts.pop_front(), parts.pop_front()) {
                    (Some(5), Some(color)) => foreground = Some(Color::Fixed(color)),
                    (Some(2), Some(red)) => match (parts.pop_front(), parts.pop_front()) {
                        (Some(green), Some(blue)) => {
                            foreground = Some(Color::RGB(red, green, blue))
                        }
                        _ => {
                            break;
                        }
                    },
                    _ => {
                        break;
                    }
                },
                Some(39) => foreground = None,
                Some(40) => background = Some(Color::Black),
                Some(41) => background = Some(Color::Red),
                Some(42) => background = Some(Color::Green),
                Some(43) => background = Some(Color::Yellow),
                Some(44) => background = Some(Color::Blue),
                Some(45) => background = Some(Color::Purple),
                Some(46) => background = Some(Color::Cyan),
                Some(47) => background = Some(Color::White),
                Some(48) => match (parts.pop_front(), parts.pop_front()) {
                    (Some(5), Some(color)) => background = Some(Color::Fixed(color)),
                    (Some(2), Some(red)) => match (parts.pop_front(), parts.pop_front()) {
                        (Some(green), Some(blue)) => {
                            background = Some(Color::RGB(red, green, blue))
                        }
                        _ => {
                            break;
                        }
                    },
                    _ => {
                        break;
                    }
                },
                Some(49) => background = None,
                Some(_) | None => {
                    break;
                }
            }
        }

        Some(Style {
            foreground,
            background,
            font_style,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, FontStyle, Style};

    fn assert_style(
        code: &str,
        foreground: Option<Color>,
        background: Option<Color>,
        font_style: FontStyle,
    ) {
        let style = Style::from_ansi_sequence(code).unwrap();
        assert_eq!(foreground, style.foreground);
        assert_eq!(font_style, style.font_style);
    }

    #[test]
    fn parse_simple() {
        assert_style("31", Some(Color::Red), None, FontStyle::default());
        assert_style("47", None, Some(Color::White), FontStyle::default());
        assert_style(
            "32;40",
            Some(Color::Green),
            Some(Color::Black),
            FontStyle::default(),
        );
    }

    #[test]
    fn parse_font_style() {
        assert_style("00;31", Some(Color::Red), None, FontStyle::default());
        assert_style("03;34", Some(Color::Blue), None, FontStyle::italic());
        assert_style("01;36", Some(Color::Cyan), None, FontStyle::bold());
        let italic_and_bold = FontStyle {
            bold: true,
            italic: true,
            underline: false,
        };
        assert_style("01;03", None, None, italic_and_bold);
    }

    #[test]
    fn parse_font_style_backwards() {
        assert_style("34;03", Some(Color::Blue), None, FontStyle::italic());
        assert_style("36;01", Some(Color::Cyan), None, FontStyle::bold());
        assert_style("31;00", Some(Color::Red), None, FontStyle::default());
    }

    #[test]
    fn parse_8_bit_colors() {
        assert_style(
            "38;5;115",
            Some(Color::Fixed(115)),
            None,
            FontStyle::default(),
        );
        assert_style(
            "00;38;5;115",
            Some(Color::Fixed(115)),
            None,
            FontStyle::default(),
        );
        assert_style(
            "01;38;5;119",
            Some(Color::Fixed(119)),
            None,
            FontStyle::bold(),
        );
        assert_style(
            "38;5;119;01",
            Some(Color::Fixed(119)),
            None,
            FontStyle::bold(),
        );
    }

    #[test]
    fn parse_24_bit_colors() {
        assert_style(
            "38;2;115;3;100",
            Some(Color::RGB(115, 3, 100)),
            None,
            FontStyle::default(),
        );
        assert_style(
            "38;2;115;3;100;3",
            Some(Color::RGB(115, 3, 100)),
            None,
            FontStyle::italic(),
        );
    }
}
