use num_format::{Locale, ToFormattedString};

pub fn fmt_token(value: f64) -> String {
    fmt_ccy(value, "")
}

pub fn fmt_usd(value: f64) -> String {
    fmt_ccy(value, "$")
}

fn fmt_ccy(value: f64, ccy: &str) -> String {
    let rounded = (value * 100.0).round() / 100.0;

    let int_part = rounded.trunc() as u64;
    let frac_part = ((rounded.fract() * 100.0).round()) as u64;

    format!(
        "{}{}.{:02}",
        ccy,
        int_part.to_formatted_string(&Locale::en),
        frac_part
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt() {
        let value: f64 = 123456789.1234;
        assert_eq!("123,456,789.12", fmt_token(value));
        assert_eq!("$123,456,789.12", fmt_usd(value));
    }
}
