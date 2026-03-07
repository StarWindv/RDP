use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;
use crate::*;

#[derive(Error, Debug, Clone)]
#[error("Invalid hex color format: {0}")]
pub struct HexColorFormatError(String);

pub struct Colors;

lazy_static! {
    static ref HEX_COLOR_REGEX: Regex = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
    static ref COLOR_CONSTANTS: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("Red", RED);
        map.insert("Orange", ORANGE);
        map.insert("Yellow", YELLOW);
        map.insert("Green", GREEN);
        map.insert("Cyan", CYAN);
        map.insert("Blue", BLUE);
        map.insert("Purple", PURPLE);

        map.insert("Bold", BOLD);
        map.insert("CleanBold", CLEAN_BOLD);

        map.insert("Underline", UNDERLINE);
        map.insert("CleanUnderline", CLEAN_UNDERLINE);

        map.insert("LastStart", LAST_START);

        map.insert("HyperStart", HYPER_START);
        map.insert("HyperText", HYPER_TEXT);
        map.insert("HyperEnd", HYPER_END);

        map.insert("ItalicStart", ITALIC_START);
        map.insert("ItalicEnd", ITALIC_END);

        map.insert("Reset", RESET_ALL);
        map.insert("ResetFg", RESET_FG);
        map.insert("ResetBg", RESET_BG);
        map
    };
}

impl Colors {
    pub fn is_valid_hex_color(color_str: &str) -> bool {
        HEX_COLOR_REGEX.is_match(color_str)
    }

    pub fn hex_to_rgb(hex_color: &str) -> Result<[u8; 3], HexColorFormatError> {
        let hex_color = hex_color.trim_start_matches('#');
        if !Self::is_valid_hex_color(hex_color) {
            return Err(HexColorFormatError(hex_color.to_string()));
        }

        match hex_color.len() {
            3 => {
                let mut rgb = [0; 3];
                for (i, c) in hex_color.chars().enumerate() {
                    let hex_str = format!("{}{}", c, c);
                    rgb[i] = u8::from_str_radix(&hex_str, 16)
                        .map_err(|_| HexColorFormatError(hex_color.to_string()))?;
                }
                Ok(rgb)
            }
            6 => {
                let r = u8::from_str_radix(&hex_color[0..2], 16)
                    .map_err(|_| HexColorFormatError(hex_color.to_string()))?;
                let g = u8::from_str_radix(&hex_color[2..4], 16)
                    .map_err(|_| HexColorFormatError(hex_color.to_string()))?;
                let b = u8::from_str_radix(&hex_color[4..6], 16)
                    .map_err(|_| HexColorFormatError(hex_color.to_string()))?;
                Ok([r, g, b])
            }
            _ => Err(HexColorFormatError(hex_color.to_string())),
        }
    }

    // 生成前景颜色的ANSI码
    pub fn front_from_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1B[38;2;{};{};{}m", r, g, b)
    }

    pub fn front_from_rgb_str(r: &str, g: &str, b: &str) -> String {
        format!("\x1B[38;2;{};{};{}m", r, g, b)
    }

    pub fn front_from_hex(hex_color: &str) -> Result<String, HexColorFormatError> {
        let [r, g, b] = Self::hex_to_rgb(hex_color)?;
        Ok(Self::front_from_rgb(r, g, b))
    }

    // 生成背景颜色的ANSI码
    pub fn background_from_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1B[48;2;{};{};{}m", r, g, b)
    }

    pub fn background_from_rgb_str(r: &str, g: &str, b: &str) -> String {
        format!("\x1B[48;2;{};{};{}m", r, g, b)
    }

    pub fn background_from_hex(hex_color: &str) -> Result<String, HexColorFormatError> {
        let [r, g, b] = Self::hex_to_rgb(hex_color)?;
        Ok(Self::background_from_rgb(r, g, b))
    }

    pub fn getattr(name: &str) -> &'static str {
        COLOR_CONSTANTS.get(name).copied().unwrap_or("")
    }
}
