use crate::error::{AppError, Result};

pub fn parse_cny_to_minor(input: &str) -> Result<i64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::new("error.amountInvalid"));
    }
    if trimmed.starts_with('-') || trimmed.starts_with('+') {
        return Err(AppError::new("error.amountInvalid"));
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() > 2 {
        return Err(AppError::new("error.amountInvalid"));
    }
    let whole = parts[0];
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::new("error.amountInvalid"));
    }
    let frac = if parts.len() == 2 { parts[1] } else { "" };
    if frac.len() > 2 || !frac.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::new("error.amountInvalid"));
    }
    let whole_val: i64 = whole
        .parse()
        .map_err(|_| AppError::new("error.amountInvalid"))?;
    let frac_padded = format!("{frac:0<2}");
    let frac_val: i64 = if frac_padded.is_empty() {
        0
    } else {
        frac_padded
            .parse()
            .map_err(|_| AppError::new("error.amountInvalid"))?
    };
    whole_val
        .checked_mul(100)
        .and_then(|v| v.checked_add(frac_val))
        .ok_or_else(|| AppError::new("error.amountOverflow"))
}

pub fn format_minor_plain(minor: i64) -> String {
    let sign = if minor < 0 { "-" } else { "" };
    let abs = minor.abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_decimals() {
        assert_eq!(parse_cny_to_minor("123.45").unwrap(), 12345);
        assert_eq!(parse_cny_to_minor("0.01").unwrap(), 1);
        assert_eq!(parse_cny_to_minor("0.1").unwrap(), 10);
        assert_eq!(parse_cny_to_minor("12").unwrap(), 1200);
    }

    #[test]
    fn parse_rejects_invalid() {
        assert!(parse_cny_to_minor("").is_err());
        assert!(parse_cny_to_minor("-1").is_err());
        assert!(parse_cny_to_minor("1.234").is_err());
        assert!(parse_cny_to_minor("12.3.4").is_err());
        assert!(parse_cny_to_minor("abc").is_err());
    }

    #[test]
    fn format_roundtrip_shape() {
        assert_eq!(format_minor_plain(12345), "123.45");
        assert_eq!(format_minor_plain(1), "0.01");
        assert_eq!(format_minor_plain(0), "0.00");
    }
}
