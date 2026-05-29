pub fn badge_color(pct: f64) -> &'static str {
    match pct as u32 {
        90..=100 => "brightgreen",
        80..=89 => "green",
        70..=79 => "yellowgreen",
        60..=69 => "yellow",
        50..=59 => "orange",
        _ => "red",
    }
}
