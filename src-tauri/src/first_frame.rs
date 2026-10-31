//! The window's colour before the page has drawn: the page's own background as near as Rust can tell, so Haunt opens on
//! its own colour (with the ghost app/assets/css/main.css draws over it) instead of a white flash. The
//! page's exact colours take over once it loads.
//!
//! theme.json's `roles.background` when it names one; else light or dark (theme.json's mode, the profile's, or the
//! system's), dark on Tailwind's 900 of a named neutral colour and on neutral 900 otherwise, as Nuxt UI's page is.

use crate::settings::Appearance;

type Rgb = (u8, u8, u8);

const LIGHT: Rgb = (255, 255, 255);
/// Tailwind's 900 of each neutral colour, the page in dark mode; neutral's for anything else (a theme's, a hex).
const DARK: [(&str, Rgb); 5] = [
    ("neutral", (23, 23, 23)),
    ("slate", (15, 23, 43)),
    ("gray", (16, 24, 40)),
    ("zinc", (24, 24, 27)),
    ("stone", (28, 25, 23)),
];

/// The colour to open the window on; None for a see-through window (theme.json's opacity, Linux).
pub fn color(profile: &Appearance) -> Option<tauri::window::Color> {
    if crate::theme_file::transparent_window() {
        return None;
    }
    let file = crate::theme_file::current().theme;
    let (r, g, b) = pick(profile, file.as_ref().map(|file| (&file.look, file.roles.background.as_str())), system_dark);
    Some(tauri::window::Color(r, g, b, 255))
}

fn pick(profile: &Appearance, file: Option<(&Appearance, &str)>, system_dark: impl FnOnce() -> bool) -> Rgb {
    if let Some(background) = file.and_then(|(_, background)| hex(background)) {
        return background;
    }
    let file_look = file.map(|(look, _)| look);
    let mode = file_look.map(|look| look.mode.as_str()).filter(|mode| !mode.is_empty()).unwrap_or(&profile.mode);
    let dark = match mode {
        "dark" => true,
        "light" => false,
        _ => system_dark(),
    };
    if !dark {
        return LIGHT;
    }
    let neutral = [file_look, Some(profile)]
        .into_iter()
        .flatten()
        .flat_map(|look| [look.dark_neutral.as_str(), look.neutral.as_str()])
        .find(|name| !name.is_empty())
        .unwrap_or_default();
    DARK.iter().find(|(name, _)| *name == neutral).map_or(DARK[0].1, |(_, rgb)| *rgb)
}

/// `#rrggbb` as its colour.
fn hex(text: &str) -> Option<Rgb> {
    let digits = text.strip_prefix('#').filter(|digits| digits.len() == 6 && digits.is_ascii())?;
    let part = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok();
    Some((part(0)?, part(2)?, part(4)?))
}

/// Whether the system is dark now: the desktop's portal or gsettings (Linux, dark until it has answered), Windows' app
/// mode, macOS's appearance.
fn system_dark() -> bool {
    if cfg!(target_os = "linux") {
        return crate::desktop::current().dark.unwrap_or(true);
    }
    let mut command = if cfg!(windows) {
        let mut command = crate::process::command("reg");
        command.args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ]);
        command
    } else {
        let mut command = crate::process::command("defaults");
        command.args(["read", "-g", "AppleInterfaceStyle"]);
        command
    };
    command.stdin(std::process::Stdio::null());
    crate::startup::hide_console(&mut command);
    let Ok(output) = command.output() else { return false };
    let said = String::from_utf8_lossy(&output.stdout);
    if cfg!(windows) {
        said.split_whitespace().last() == Some("0x0")
    } else {
        said.trim() == "Dark"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(mode: &str, neutral: &str) -> Appearance {
        Appearance { mode: mode.into(), neutral: neutral.into(), ..Appearance::default() }
    }

    #[test]
    fn opens_on_the_pages_own_colour() {
        let never = || panic!("the system was asked");
        assert_eq!(pick(&look("light", ""), None, never), LIGHT);
        assert_eq!(pick(&look("dark", ""), None, never), (23, 23, 23));
        assert_eq!(pick(&look("dark", "zinc"), None, never), (24, 24, 27));
        assert_eq!(pick(&look("dark", "#123456"), None, never), (23, 23, 23));
        assert_eq!(pick(&look("system", ""), None, || true), (23, 23, 23));
        assert_eq!(pick(&look("", ""), None, || false), LIGHT);
        // theme.json: its page colour first, then its mode and neutral over the profile's.
        let file = look("dark", "slate");
        assert_eq!(pick(&look("light", ""), Some((&file, "#141218")), never), (20, 18, 24));
        assert_eq!(pick(&look("light", "zinc"), Some((&file, "")), never), (15, 23, 43));
        assert_eq!(pick(&look("dark", "stone"), Some((&look("", ""), "nope")), never), (28, 25, 23));
    }
}
