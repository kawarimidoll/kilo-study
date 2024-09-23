pub fn is_number_string(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    let mut chars = word.chars();

    if word.starts_with("0x") || word.starts_with("0X") {
        // consume the first two characters
        let _ = chars.next();
        let _ = chars.next();
        return chars.all(|c| c.is_ascii_hexdigit());
    } else if word.starts_with("0b") || word.starts_with("0B") {
        // consume the first two characters
        let _ = chars.next();
        let _ = chars.next();
        return chars.all(|c| c.is_digit(2));
    } else if word.starts_with("0o") || word.starts_with("0O") {
        // consume the first two characters
        let _ = chars.next();
        let _ = chars.next();
        return chars.all(|c| c.is_digit(8));
    }

    // Check if the first character is a digit
    let first = chars.next().unwrap();
    if !first.is_ascii_digit() {
        return false;
    }

    let mut seen_dot = false;
    let mut seen_e = false;
    let mut prev_was_digit = true;
    for char in chars {
        match char {
            '0'..='9' => prev_was_digit = true,
            '_' => {
                if !prev_was_digit {
                    // underscores can't be next to each other
                    return false;
                }
                prev_was_digit = false;
            }
            '.' => {
                if seen_dot || seen_e || !prev_was_digit {
                    // can't have more than one dot or an e after a dot
                    return false;
                }
                seen_dot = true;
                prev_was_digit = false;
            }
            'e' | 'E' => {
                if seen_e || !prev_was_digit {
                    // can't have more than one e or an e after a dot
                    return false;
                }
                seen_e = true;
                prev_was_digit = false;
            }
            _ => {
                return false;
            }
        }
    }
    // The last character must be a digit
    prev_was_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_number_string() {
        // Valid numbers:

        assert_eq!(is_number_string("1"), true);
        assert_eq!(is_number_string("2"), true);
        assert_eq!(is_number_string("3"), true);
        assert_eq!(is_number_string("4"), true);
        assert_eq!(is_number_string("5"), true);
        assert_eq!(is_number_string("6"), true);
        assert_eq!(is_number_string("7"), true);
        assert_eq!(is_number_string("8"), true);
        assert_eq!(is_number_string("9"), true);
        assert_eq!(is_number_string("0"), true);
        assert_eq!(is_number_string("100"), true);
        assert_eq!(is_number_string("1234567"), true);
        assert_eq!(is_number_string("1.0"), true);
        assert_eq!(is_number_string("2.0"), true);
        assert_eq!(is_number_string("3.0"), true);
        assert_eq!(is_number_string("0.0"), true);
        assert_eq!(is_number_string("1e10"), true);
        assert_eq!(is_number_string("20e50"), true);
        assert_eq!(is_number_string("10.3e5"), true);
        assert_eq!(is_number_string("1_00"), true);
        assert_eq!(is_number_string("1_000_1"), true);
        assert_eq!(is_number_string("1_000_000_000"), true);
        assert_eq!(is_number_string("0x1"), true);
        assert_eq!(is_number_string("0X2"), true);
        assert_eq!(is_number_string("0b1"), true);
        assert_eq!(is_number_string("0B0"), true);
        assert_eq!(is_number_string("0X10F"), true);
        assert_eq!(is_number_string("0o1"), true);

        // Invalid numbers:

        assert_eq!(is_number_string("1a"), false);
        assert_eq!(is_number_string("2b"), false);
        assert_eq!(is_number_string("3c"), false);
        assert_eq!(is_number_string("4d"), false);
        assert_eq!(is_number_string("5e"), false);
        assert_eq!(is_number_string("6f"), false);
        assert_eq!(is_number_string("7g"), false);
        assert_eq!(is_number_string("8h"), false);
        assert_eq!(is_number_string("9i"), false);
        assert_eq!(is_number_string("0j"), false);
        assert_eq!(is_number_string("100a200b300c400d500"), false);
        assert_eq!(is_number_string("u32"), false);
        assert_eq!(is_number_string("i8"), false);
        assert_eq!(is_number_string("f64"), false);
        assert_eq!(is_number_string("1.1.2"), false);
        assert_eq!(is_number_string("2.2.3"), false);
        assert_eq!(is_number_string("3.3.4"), false);
        assert_eq!(is_number_string("4.4.5"), false);
        assert_eq!(is_number_string("5.5.6"), false);
        assert_eq!(is_number_string("6.6.7"), false);
        assert_eq!(is_number_string("7.7.8"), false);
        assert_eq!(is_number_string("8.8.9"), false);
        assert_eq!(is_number_string("9.9.0"), false);
        assert_eq!(is_number_string("0.0.1"), false);
        assert_eq!(is_number_string("1e"), false);
        assert_eq!(is_number_string("e3"), false);
        assert_eq!(is_number_string("e"), false);
        assert_eq!(is_number_string("1e2e"), false);
        assert_eq!(is_number_string("5.8e10.1"), false);
        assert_eq!(is_number_string("_100_1"), false);
        assert_eq!(is_number_string("100_"), false);
        assert_eq!(is_number_string("1_00_"), false);
        assert_eq!(is_number_string("_"), false);
        assert_eq!(is_number_string("0b102"), false);
        assert_eq!(is_number_string("0x1G"), false);
        assert_eq!(is_number_string("1o108"), false);
        assert_eq!(is_number_string("0xxx"), false);
    }
}
