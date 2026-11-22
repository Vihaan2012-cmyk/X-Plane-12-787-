use crate::font::measure_text;

pub fn ellipsize(text: &str, size: f32, max_w: f32) -> String {
    if measure_text(text, size).0 <= max_w {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut n = chars.len();
    while n > 0 {
        let mut s: String = chars[..n].iter().collect();
        let trimmed = s.trim_end().len();
        s.truncate(trimmed);
        s.push_str("..");
        if measure_text(&s, size).0 <= max_w {
            return s;
        }
        n -= 1;
    }
    String::new()
}

pub fn fit_size(text: &str, max: f32, min: f32, max_w: f32) -> f32 {
    let mut s = max;
    while s > min && measure_text(text, s).0 > max_w {
        s -= 0.5;
    }
    s.max(min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_unchanged() {
        assert_eq!(ellipsize("abc", 10.0, 100.0), "abc");
    }

    #[test]
    fn long_text_fits_with_dots() {
        let s = ellipsize("a very long label indeed", 10.0, 60.0);
        assert!(s.ends_with(".."));
        assert!(measure_text(&s, 10.0).0 <= 60.0);
    }

    #[test]
    fn size_shrinks_until_fit() {
        assert_eq!(fit_size("abcd", 10.0, 6.0, 1000.0), 10.0);
        assert!(fit_size("abcdefghij", 10.0, 6.0, 40.0) < 10.0);
    }
}
