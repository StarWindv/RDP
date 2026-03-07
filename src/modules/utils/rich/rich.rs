use chrono::{Local};
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use lazy_static::lazy_static;
use super::colors;

lazy_static! {
    static ref FORMATTER: RwLock<String> = RwLock::new("[%Y-%m-%d %H:%M:%S]".to_string());
}


pub struct Rich {}


#[allow(dead_code)]
impl Rich {
    pub fn set_formatter(format: &str) {
        let mut f = FORMATTER.write().expect("Failed to lock formatter for writing");
        *f = format.to_string();
    }

    pub fn extract_bracket_content(
        input: &str,
        left: char,
        right: char,
        min_length: usize,
        max_length: usize,
        clean_blank: bool,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();
        let mut left_bracket_indices = VecDeque::new();

        if input.is_empty() {
            return result;
        }

        let chars: Vec<char> = input.chars().collect();
        for i in 0..chars.len() {
            let current_char = chars[i];
            // 跳过转义的括号（前面有\的）
            if i > 0 && chars[i-1] == '\\' {
                continue;
            }

            if current_char == left {
                left_bracket_indices.push_back(i);
            } else if current_char == right && !left_bracket_indices.is_empty() {
                let left_index = left_bracket_indices.pop_back().unwrap();
                // 提取括号内的内容
                let content: String = chars[(left_index + 1)..i].iter().collect();
                let mut cleaned_content = content.clone();
                if clean_blank {
                    cleaned_content = cleaned_content.replace(|c: char| c.is_whitespace(), "");
                }

                // 检查内容长度，并且内容里不能有左括号
                if !content.is_empty() && !content.contains(left) {
                    if content.len() >= min_length && content.len() <= max_length {
                        let key: String = chars[left_index..=i].iter().collect();
                        result.insert(key, cleaned_content);
                    }
                }
            }
        }

        result
    }

    pub fn color_match(input: &str) -> HashMap<String, String> {
        Rich::extract_bracket_content(input, '[', ']', 1, 14, true)
    }

    pub fn tag_match(input: &str) -> HashMap<String, String> {
        Rich::extract_bracket_content(input, '<', '>', 3, 10, true)
    }

    pub fn counting(source: &str, target: &str, expected_count: usize) -> bool {
        source.matches(target).count() == expected_count
    }

    pub fn parse_colors(mut input: String, clean: bool) -> String {
        let color_tags = Rich::color_match(&input);

        for (tag, content) in color_tags {
            if content == "/" {
                input = input.replace(&tag, super::RESET_ALL);
                continue;
            }
            let (is_background, color_value) = if content.starts_with("bg-") {
                (true, &content[3..])
            } else {
                (false, content.as_str())
            };

            let color_code = Self::value_process(color_value, is_background)
                .unwrap_or_else(|| String::new());
            if !color_code.is_empty() && !clean {
                input = input.replace(&tag, &color_code);
            } else if clean {
                input = input.replace(&tag, "");
            }
        }
        input + super::RESET_ALL
    }

    fn value_process(value: &str, is_background: bool) -> Option<String> {
        if value.starts_with('#') {
            if is_background {
                colors::Colors::background_from_hex(value).ok()
            } else {
                colors::Colors::front_from_hex(value).ok()
            }
        } else if value.split(',').count() >= 3 {
            let parts: Vec<&str> = value.split(',').collect();
            let extract_component = |idx: usize| -> &str {
                parts.get(idx)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("0")
            };
            let color_code = if is_background {
                colors::Colors::background_from_rgb_str(
                    extract_component(0),
                    extract_component(1),
                    extract_component(2)
                )
            } else {
                colors::Colors::front_from_rgb_str(
                    extract_component(0),
                    extract_component(1),
                    extract_component(2)
                )
            };

            Some(color_code)
        } else {
            None
        }
    }

    pub fn parse_tags(mut input: String) -> String {
        let tags = Rich::tag_match(&input);
        for (tag, content) in tags {
            if content == "TIMESTAMP" {
                // 替换为时间戳
                let now = Local::now();
                let format = FORMATTER.read().expect("Failed to lock formatter for reading");
                let timestamp = now.format(&format).to_string();
                input = input.replace(&tag, &timestamp);
            } else {
                // 尝试从colors里获取对应的常量
                let value = colors::Colors::getattr(&content);
                if value != "" {
                    input = input.replace(&tag, value);
                }
            }
        }
        // 最后加上重置码
        input + super::RESET_ALL
    }

    pub fn outline(input: &str) {
        let processed = Rich::process(input);
        println!("{}", processed);
    }

    pub fn outline_empty() {
        println!();
    }

    pub fn out(input: &str) {
        let processed = Rich::process(input);
        print!("{}", processed);
    }

    pub fn out_empty() {
        print!("");
    }

    pub fn process(input: &str) -> String {
        Rich::parse_tags(Rich::parse_colors(input.to_string(), false))
    }

    pub fn clean(input: &str) -> String {
        Rich::parse_tags(Rich::parse_colors(input.to_string(), true))
    }
}
