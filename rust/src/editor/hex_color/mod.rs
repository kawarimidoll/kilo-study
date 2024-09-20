use crossterm::style::Color;

#[derive(Eq, PartialEq, Debug)]
pub struct HexColor {
    r: u8,
    g: u8,
    b: u8,
}

#[allow(dead_code)]
impl HexColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn from(hex: &str) -> Result<Self, String> {
        if hex.len() == 4 && hex.starts_with('#') {
            // #RGB
            let r = Self::parse_hex(&hex[1..2].repeat(2))?;
            let g = Self::parse_hex(&hex[2..3].repeat(2))?;
            let b = Self::parse_hex(&hex[3..4].repeat(2))?;
            Ok(Self { r, g, b })
        } else if hex.len() == 7 && hex.starts_with('#') {
            // #RRGGBB
            let r = Self::parse_hex(&hex[1..3])?;
            let g = Self::parse_hex(&hex[3..5])?;
            let b = Self::parse_hex(&hex[5..7])?;
            Ok(Self { r, g, b })
        } else {
            Err("Invalid hex format".to_string())
        }
    }

    fn parse_hex(str_val: &str) -> Result<u8, &str> {
        u8::from_str_radix(str_val, 16).map_err(|_| "Invalid hex format")
    }

    pub fn as_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_color(&self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

// impl From<HexColor> for Color {
//     fn from(hex: HexColor) -> Self {
//         let HexColor { r, g, b } = hex;
//         Color::Rgb { r, g, b }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(HexColor::parse_hex("0A"), Ok(10));
        assert_eq!(HexColor::parse_hex("KA"), Err("Invalid hex format"));
    }

    #[test]
    fn test_from() {
        let hex = HexColor::from("#4A7");
        assert_eq!(
            hex,
            Ok(HexColor {
                r: 68,
                g: 170,
                b: 119
            })
        );
        let hex = HexColor::from("#14B9C3");
        assert_eq!(
            hex,
            Ok(HexColor {
                r: 20,
                g: 185,
                b: 195
            })
        );
        let hex = HexColor::from("14B9C3");
        assert_eq!(hex, Err("Invalid hex format".to_string()));
        let hex = HexColor::from("#C3");
        assert_eq!(hex, Err("Invalid hex format".to_string()));
        let hex = HexColor::from("#K4B9C3");
        assert_eq!(hex, Err("Invalid hex format".to_string()));
    }

    #[test]
    fn test_as_string() {
        let hex = HexColor::new(100, 39, 9);
        assert_eq!(hex.as_string(), "#642709");
        let hex = HexColor::from("#14B9C3").unwrap();
        assert_eq!(hex.as_string(), "#14B9C3");
        let hex = HexColor::from("#804").unwrap();
        assert_eq!(hex.as_string(), "#880044");
    }
}
