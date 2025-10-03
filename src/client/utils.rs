use num_format::{Locale, ToFormattedString};

pub fn fmt_usd(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;

    let int_part = rounded.trunc() as u64;
    let frac_part = ((rounded.fract() * 100.0).round()) as u64;

    format!(
        "${}.{:02}",
        int_part.to_formatted_string(&Locale::en),
        frac_part
    )
}
