use std::collections::HashSet;

use crate::error::{AppError, AppResult};

const MIN_PASSWORD_LEN: usize = 10;
const MIN_SCORE: u8 = 3;
const MIN_ENTROPY_BITS: f64 = 50.0;

const COMMON_TERMS: &[&str] = &[
    "password", "passw0rd", "qwerty", "letmein", "welcome", "admin", "login", "secret", "blevel",
    "tracker",
];

#[derive(Debug, Clone, PartialEq)]
pub struct PasswordStrength {
    pub score: u8,
    pub entropy_bits: f64,
    pub feedback: Vec<String>,
}

pub fn validate_new_password(password: &str) -> AppResult<()> {
    let strength = estimate_password_strength(password);
    if strength.score >= MIN_SCORE && strength.entropy_bits >= MIN_ENTROPY_BITS {
        return Ok(());
    }

    let reason = strength
        .feedback
        .first()
        .cloned()
        .unwrap_or_else(|| "use a longer, less predictable passphrase".into());
    Err(AppError::BadRequest(format!(
        "Password is too weak: {reason}."
    )))
}

pub fn estimate_password_strength(password: &str) -> PasswordStrength {
    let char_count = password.chars().count();
    let mut feedback = Vec::new();

    if char_count < MIN_PASSWORD_LEN {
        feedback.push(format!("use at least {MIN_PASSWORD_LEN} characters"));
    }

    let lower = password.chars().any(|c| c.is_ascii_lowercase());
    let upper = password.chars().any(|c| c.is_ascii_uppercase());
    let digit = password.chars().any(|c| c.is_ascii_digit());
    let symbol = password
        .chars()
        .any(|c| c.is_ascii_punctuation() || c.is_ascii_whitespace());
    let other = !password.is_ascii();

    let mut alphabet_size = 0u32;
    if lower {
        alphabet_size += 26;
    }
    if upper {
        alphabet_size += 26;
    }
    if digit {
        alphabet_size += 10;
    }
    if symbol {
        alphabet_size += 33;
    }
    if other {
        alphabet_size += 64;
    }
    if alphabet_size == 0 {
        alphabet_size = 1;
    }

    let mut entropy_bits = char_count as f64 * (alphabet_size as f64).log2();

    let normalized = password.to_ascii_lowercase();
    if COMMON_TERMS.iter().any(|term| normalized.contains(term)) {
        entropy_bits -= 30.0;
        feedback.push("avoid common words or app-specific terms".into());
    }
    if contains_keyboard_or_numeric_sequence(&normalized) {
        entropy_bits -= 20.0;
        feedback.push("avoid keyboard or numeric sequences".into());
    }

    let unique_chars = password.chars().collect::<HashSet<_>>().len();
    if unique_chars < 5 && char_count >= MIN_PASSWORD_LEN {
        entropy_bits -= 30.0;
        feedback.push("use more unique characters".into());
    }

    let longest_run = longest_repeated_run(password);
    if longest_run >= 4 {
        entropy_bits -= ((longest_run - 3) * 8) as f64;
        feedback.push("avoid long repeated-character runs".into());
    }

    if is_repeated_pattern(password) {
        entropy_bits -= 35.0;
        feedback.push("avoid repeating the same short pattern".into());
    }

    entropy_bits = entropy_bits.max(0.0);
    if entropy_bits < MIN_ENTROPY_BITS && feedback.is_empty() {
        feedback.push("make it longer or less predictable".into());
    }

    let score = match entropy_bits {
        e if e >= 80.0 && char_count >= 16 => 4,
        e if e >= MIN_ENTROPY_BITS && char_count >= MIN_PASSWORD_LEN => 3,
        e if e >= 35.0 => 2,
        e if e >= 20.0 => 1,
        _ => 0,
    };

    PasswordStrength {
        score,
        entropy_bits,
        feedback,
    }
}

fn longest_repeated_run(password: &str) -> usize {
    let mut previous = None;
    let mut current = 0usize;
    let mut longest = 0usize;
    for c in password.chars() {
        if Some(c) == previous {
            current += 1;
        } else {
            previous = Some(c);
            current = 1;
        }
        longest = longest.max(current);
    }
    longest
}

fn is_repeated_pattern(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    let len = chars.len();
    if len < MIN_PASSWORD_LEN {
        return false;
    }
    for pattern_len in 1..=(len / 2) {
        if len % pattern_len != 0 {
            continue;
        }
        if chars
            .chunks(pattern_len)
            .all(|chunk| chunk == &chars[..pattern_len])
        {
            return true;
        }
    }
    false
}

fn contains_keyboard_or_numeric_sequence(normalized: &str) -> bool {
    const SEQUENCES: &[&str] = &[
        "0123456789",
        "9876543210",
        "abcdefghijklmnopqrstuvwxyz",
        "zyxwvutsrqponmlkjihgfedcba",
        "qwertyuiop",
        "poiuytrewq",
        "asdfghjkl",
        "lkjhgfdsa",
        "zxcvbnm",
        "mnbvcxz",
    ];

    SEQUENCES.iter().any(|seq| {
        seq.as_bytes()
            .windows(5)
            .any(|window| normalized.contains(std::str::from_utf8(window).unwrap_or_default()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_passwords() {
        let err = validate_new_password("Short7!").unwrap_err();
        assert!(err.user_message().contains("at least 10"));
    }

    #[test]
    fn rejects_common_predictable_passwords() {
        let strength = estimate_password_strength("password123456!");
        assert!(strength.score < MIN_SCORE);
        assert!(validate_new_password("password123456!").is_err());
    }

    #[test]
    fn rejects_repeated_patterns() {
        let strength = estimate_password_strength("Ab1!Ab1!Ab1!");
        assert!(strength.score < MIN_SCORE);
        assert!(validate_new_password("Ab1!Ab1!Ab1!").is_err());
    }

    #[test]
    fn accepts_long_passphrases() {
        assert!(validate_new_password("olive-piano-7-cliffside-music").is_ok());
        assert!(validate_new_password("dev-skip-not-for-production-x7q2").is_ok());
    }
}
