use ratatui::style::Color;

use crate::config::ThemeColors;

#[derive(Clone)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub panel: Color,
    pub accent: Color,
    pub title: Color,
    pub text: Color,
    pub dim: Color,
    pub ok: Color,
    pub warn: Color,
    pub danger: Color,
}

pub fn dark() -> Theme {
    Theme {
        name: "dark",
        bg: Color::Rgb(18, 22, 32),
        panel: Color::Rgb(24, 29, 42),
        accent: Color::Rgb(88, 176, 255),
        title: Color::Rgb(121, 216, 255),
        text: Color::Rgb(214, 221, 232),
        dim: Color::Rgb(112, 122, 140),
        ok: Color::Rgb(92, 220, 152),
        warn: Color::Rgb(255, 199, 92),
        danger: Color::Rgb(255, 94, 104),
    }
}

fn light() -> Theme {
    Theme {
        name: "light",
        bg: Color::Rgb(252, 252, 252),
        panel: Color::Rgb(240, 242, 246),
        accent: Color::Rgb(47, 108, 184),
        title: Color::Rgb(29, 78, 148),
        text: Color::Rgb(30, 34, 42),
        dim: Color::Rgb(120, 128, 140),
        ok: Color::Rgb(16, 128, 78),
        warn: Color::Rgb(180, 130, 20),
        danger: Color::Rgb(200, 40, 50),
    }
}

fn gruvbox() -> Theme {
    Theme {
        name: "gruvbox",
        bg: Color::Rgb(40, 40, 40),
        panel: Color::Rgb(60, 56, 54),
        accent: Color::Rgb(131, 165, 152),
        title: Color::Rgb(250, 189, 47),
        text: Color::Rgb(235, 219, 178),
        dim: Color::Rgb(146, 131, 116),
        ok: Color::Rgb(152, 151, 26),
        warn: Color::Rgb(214, 93, 14),
        danger: Color::Rgb(204, 36, 29),
    }
}

fn dracula() -> Theme {
    Theme {
        name: "dracula",
        bg: Color::Rgb(40, 42, 54),
        panel: Color::Rgb(52, 54, 68),
        accent: Color::Rgb(189, 147, 249),
        title: Color::Rgb(255, 121, 198),
        text: Color::Rgb(248, 248, 242),
        dim: Color::Rgb(98, 114, 164),
        ok: Color::Rgb(80, 250, 123),
        warn: Color::Rgb(241, 250, 140),
        danger: Color::Rgb(255, 85, 85),
    }
}

fn solarized() -> Theme {
    Theme {
        name: "solarized",
        bg: Color::Rgb(0, 43, 54),
        panel: Color::Rgb(7, 54, 66),
        accent: Color::Rgb(38, 139, 210),
        title: Color::Rgb(42, 161, 152),
        text: Color::Rgb(131, 148, 150),
        dim: Color::Rgb(88, 110, 117),
        ok: Color::Rgb(133, 153, 0),
        warn: Color::Rgb(181, 137, 0),
        danger: Color::Rgb(220, 50, 47),
    }
}

fn nord() -> Theme {
    Theme {
        name: "nord",
        bg: Color::Rgb(46, 52, 64),
        panel: Color::Rgb(59, 66, 82),
        accent: Color::Rgb(136, 192, 208),
        title: Color::Rgb(143, 188, 187),
        text: Color::Rgb(216, 222, 233),
        dim: Color::Rgb(94, 108, 132),
        ok: Color::Rgb(163, 190, 140),
        warn: Color::Rgb(235, 203, 139),
        danger: Color::Rgb(191, 97, 106),
    }
}

pub fn from_name(name: &str) -> Option<Theme> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "dark" => dark(),
        "light" => light(),
        "gruvbox" => gruvbox(),
        "dracula" => dracula(),
        "solarized" => solarized(),
        "nord" => nord(),
        _ => return None,
    })
}

pub fn all_names() -> &'static str {
    "dark, light, gruvbox, dracula, solarized, nord"
}

pub fn resolve(name: Option<&str>, overrides: Option<&ThemeColors>) -> Theme {
    let mut theme = name.and_then(from_name).unwrap_or_else(dark);
    if let Some(c) = overrides {
        if let Some(v) = c.bg.as_deref().and_then(parse_hex) {
            theme.bg = v;
        }
        if let Some(v) = c.panel.as_deref().and_then(parse_hex) {
            theme.panel = v;
        }
        if let Some(v) = c.accent.as_deref().and_then(parse_hex) {
            theme.accent = v;
        }
        if let Some(v) = c.title.as_deref().and_then(parse_hex) {
            theme.title = v;
        }
        if let Some(v) = c.text.as_deref().and_then(parse_hex) {
            theme.text = v;
        }
        if let Some(v) = c.dim.as_deref().and_then(parse_hex) {
            theme.dim = v;
        }
        if let Some(v) = c.ok.as_deref().and_then(parse_hex) {
            theme.ok = v;
        }
        if let Some(v) = c.warn.as_deref().and_then(parse_hex) {
            theme.warn = v;
        }
        if let Some(v) = c.danger.as_deref().and_then(parse_hex) {
            theme.danger = v;
        }
    }
    theme
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().strip_prefix('#').unwrap_or(s.trim());
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_theme_names_resolve() {
        for name in ["dark", "light", "gruvbox", "dracula", "solarized", "nord"] {
            assert!(from_name(name).is_some(), "{name} should resolve");
            assert!(from_name(&name.to_uppercase()).is_some(), "{name} is case-insensitive");
        }
    }

    #[test]
    fn unknown_theme_falls_back_to_dark() {
        assert!(from_name("neon").is_none());
        assert_eq!(resolve(Some("neon"), None).name, "dark");
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(parse_hex("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex("00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex("#0000ff"), Some(Color::Rgb(0, 0, 255)));
        assert!(parse_hex("#12345").is_none());
        assert!(parse_hex("#gggggg").is_none());
    }

    #[test]
    fn overrides_replace_colors() {
        let colors = ThemeColors {
            accent: Some("#ff0000".into()),
            ..Default::default()
        };
        let theme = resolve(Some("dark"), Some(&colors));
        assert_eq!(theme.accent, Color::Rgb(255, 0, 0));
        assert_eq!(theme.bg, dark().bg);
    }
}
