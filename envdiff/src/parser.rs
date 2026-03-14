use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Parse a .env file and return an ordered map of key -> value.
pub fn parse_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    parse_env_content(&content)
}

fn parse_env_content(content: &str) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    let mut insertion_order: Vec<String> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Strip optional `export ` prefix
        let line = if let Some(rest) = line.strip_prefix("export ") {
            rest.trim_start()
        } else {
            line
        };

        // Find the `=` separator
        let eq_pos = match line.find('=') {
            Some(p) => p,
            None => {
                i += 1;
                continue;
            }
        };

        let key = line[..eq_pos].trim().to_string();
        let raw_value = line[eq_pos + 1..].trim();

        let (value, lines_consumed) = parse_value(raw_value, &lines, i)?;
        if !map.contains_key(&key) {
            insertion_order.push(key.clone());
        }
        map.insert(key, value);
        i += lines_consumed;
    }

    // Perform variable interpolation in definition order
    for key in &insertion_order {
        let val = map[key].clone();
        let interpolated = interpolate(&val, &map);
        map.insert(key.clone(), interpolated);
    }

    Ok(map)
}

/// Parse the value portion after `=`. Returns (value, number_of_lines_consumed).
fn parse_value(raw: &str, lines: &[&str], current_line: usize) -> Result<(String, usize)> {
    if raw.is_empty() {
        return Ok((String::new(), 1));
    }

    let quote_char = raw.as_bytes()[0];

    // Handle quoted values (double or single)
    if quote_char == b'"' || quote_char == b'\'' {
        let after_open = &raw[1..];

        // Check if closing quote is on the same line
        if let Some(close_pos) = find_unescaped_quote(after_open, quote_char) {
            let val = after_open[..close_pos].to_string();
            let val = if quote_char == b'"' {
                unescape_double_quoted(&val)
            } else {
                val
            };
            return Ok((val, 1));
        }

        // Multiline quoted value
        let mut buf = after_open.to_string();
        let mut j = current_line + 1;
        while j < lines.len() {
            buf.push('\n');
            let next_line = lines[j];
            if let Some(close_pos) = find_unescaped_quote(next_line, quote_char) {
                buf.push_str(&next_line[..close_pos]);
                let val = if quote_char == b'"' {
                    unescape_double_quoted(&buf)
                } else {
                    buf
                };
                return Ok((val, j - current_line + 1));
            }
            buf.push_str(next_line);
            j += 1;
        }

        // No closing quote found -- treat everything collected as the value
        let val = if quote_char == b'"' {
            unescape_double_quoted(&buf)
        } else {
            buf
        };
        return Ok((val, j - current_line));
    }

    // Unquoted value: strip inline comments
    let val = if let Some(comment_pos) = raw.find(" #") {
        raw[..comment_pos].trim_end().to_string()
    } else {
        raw.to_string()
    };

    Ok((val, 1))
}

fn find_unescaped_quote(s: &str, quote: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped char
            continue;
        }
        if bytes[i] == quote {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn unescape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Interpolate $VAR and ${VAR} references using the provided map.
fn interpolate(value: &str, vars: &BTreeMap<String, String>) -> String {
    let mut result = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // ${VAR} form
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let var_name: String = chars[i + 2..i + 2 + close].iter().collect();
                    if let Some(val) = vars.get(&var_name) {
                        result.push_str(val);
                    }
                    i = i + 2 + close + 1;
                    continue;
                }
            }
            // $VAR form
            let start = i + 1;
            let mut end = start;
            while end < chars.len()
                && (chars[end].is_alphanumeric() || chars[end] == '_')
            {
                end += 1;
            }
            if end > start {
                let var_name: String = chars[start..end].iter().collect();
                if let Some(val) = vars.get(&var_name) {
                    result.push_str(val);
                }
                i = end;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
# comment
KEY1=value1
KEY2="value with spaces"
KEY3='single quoted'
export KEY4=exported
EMPTY=
"#;
        let map = parse_env_content(content).unwrap();
        assert_eq!(map["KEY1"], "value1");
        assert_eq!(map["KEY2"], "value with spaces");
        assert_eq!(map["KEY3"], "single quoted");
        assert_eq!(map["KEY4"], "exported");
        assert_eq!(map["EMPTY"], "");
    }

    #[test]
    fn test_multiline() {
        let content = "KEY=\"line1\nline2\nline3\"";
        let map = parse_env_content(content).unwrap();
        assert_eq!(map["KEY"], "line1\nline2\nline3");
    }

    #[test]
    fn test_interpolation() {
        let content = "HOST=localhost\nURL=http://$HOST:8080\nFULL=${URL}/api";
        let map = parse_env_content(content).unwrap();
        assert_eq!(map["URL"], "http://localhost:8080");
        assert_eq!(map["FULL"], "http://localhost:8080/api");
    }

    #[test]
    fn test_inline_comment() {
        let content = "KEY=value # this is a comment";
        let map = parse_env_content(content).unwrap();
        assert_eq!(map["KEY"], "value");
    }

    #[test]
    fn test_escape_sequences() {
        let content = r#"KEY="hello\nworld""#;
        let map = parse_env_content(content).unwrap();
        assert_eq!(map["KEY"], "hello\nworld");
    }
}
