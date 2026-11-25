use gen_lsp_types::*;
use jjpwrgem_parse::ast::{Document, Value};

use crate::{backend::Backend, range::Positions, transport::Sink};

/// parses `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`
fn parse_hex_color(text: &str) -> Option<Color> {
    let digits = text.strip_prefix('#')?;
    let width = match digits.len() {
        3 | 4 => 1,
        6 | 8 => 2,
        _ => return None,
    };
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut channels = [255u8; 4];
    for (channel, chunk) in channels.iter_mut().zip(digits.as_bytes().chunks(width)) {
        let value = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
        *channel = if width == 1 { value * 17 } else { value };
    }
    let [red, green, blue, alpha] = channels.map(|c| f32::from(c) / 255.0);
    Some(Color {
        red,
        green,
        blue,
        alpha,
    })
}

fn document_colors(
    document: &Document<String>,
    encoding: crate::range::PositionEncoding,
) -> Vec<ColorInformation> {
    let positions = Positions::new(document.source(), encoding);
    document
        .root()
        .traverse()
        .filter_map(|value| {
            let Value::String(range) = value else {
                return None;
            };
            let text = document.source().get(range.start..range.end)?;
            if !text.starts_with('#') {
                return None;
            }
            Some(ColorInformation {
                range: positions.range(*range),
                color: parse_hex_color(text)?,
            })
        })
        .collect()
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to 0..=255"
)]
fn channel(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn color_label(color: Color) -> String {
    let (r, g, b) = (
        channel(color.red),
        channel(color.green),
        channel(color.blue),
    );
    if color.alpha < 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}{:02x}", channel(color.alpha))
    } else {
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// one `#rrggbb[aa]` presentation replacing the requested range
pub(super) fn color_presentation(params: &ColorPresentationParams) -> Vec<ColorPresentation> {
    let label = color_label(params.color);
    vec![ColorPresentation {
        text_edit: Some(TextEdit {
            range: params.range,
            new_text: label.clone(),
        }),
        label,
        ..Default::default()
    }]
}

impl<S: Sink> Backend<S> {
    pub(super) fn document_color(&mut self, params: &DocumentColorParams) -> Vec<ColorInformation> {
        self.with_document(&params.text_document.uri, |document| {
            document_colors(document, self.position_encoding)
        })
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        let [red, green, blue, alpha] = [r, g, b, a].map(|c| f32::from(c) / 255.0);
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    #[rstest]
    #[case::three("#f00", Some(rgba(255, 0, 0, 255)))]
    #[case::four("#0f08", Some(rgba(0, 255, 0, 0x88)))]
    #[case::six("#0000ff", Some(rgba(0, 0, 255, 255)))]
    #[case::eight("#11223344", Some(rgba(0x11, 0x22, 0x33, 0x44)))]
    #[case::uppercase("#AbCdEf", Some(rgba(0xab, 0xcd, 0xef, 255)))]
    #[case::too_short("#12", None)]
    #[case::five("#12345", None)]
    #[case::not_hex("#ggg", None)]
    #[case::no_hash("ff0000", None)]
    #[case::hash_only("#", None)]
    #[case::non_ascii("#é1", None)]
    fn parses_hex_colors(#[case] text: &str, #[case] expected: Option<Color>) {
        assert_eq!(parse_hex_color(text), expected);
    }

    #[rstest]
    #[case::opaque(Color { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 }, "#ff0000")]
    #[case::translucent(Color { red: 0.0, green: 1.0, blue: 0.0, alpha: 0.5 }, "#00ff0080")]
    #[case::rounding(Color { red: 0.5, green: 0.001, blue: 0.999, alpha: 1.0 }, "#8000ff")]
    fn labels_colors(#[case] color: Color, #[case] expected: &str) {
        assert_eq!(color_label(color), expected);
    }
}
