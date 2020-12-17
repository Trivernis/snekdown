use crate::settings::format_settings::Theme;
use std::time::Instant;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Returns the css of a theme compiled from sass
pub fn get_css_for_theme(theme: Theme) -> String {
    let start = Instant::now();
    let vars = match theme {
        Theme::GitHub => include_str!("assets/light-github.scss"),
        Theme::SolarizedDark => include_str!("assets/dark-solarized.scss"),
        Theme::SolarizedLight => include_str!("assets/light-solarized.scss"),
        Theme::OceanDark => include_str!("assets/dark-ocean.scss"),
        Theme::OceanLight => include_str!("assets/light-ocean.scss"),
    };
    let style = format!("{}\n{}", vars, include_str!("assets/base.scss"));

    let css = compile_sass(&*style);

    log::debug!("Compiled style in {} ms", start.elapsed().as_millis());

    css
}

/// Returns the syntax theme for a given theme
pub fn get_code_theme_for_theme(theme: Theme) -> (syntect::highlighting::Theme, SyntaxSet) {
    lazy_static::lazy_static! { static ref PS: SyntaxSet = SyntaxSet::load_defaults_nonewlines(); }
    lazy_static::lazy_static! { static ref TS: ThemeSet = ThemeSet::load_defaults(); }

    let theme = match theme {
        Theme::GitHub => "InspiredGitHub",
        Theme::SolarizedDark => "Solarized (dark)",
        Theme::SolarizedLight => "Solarized (light)",
        Theme::OceanDark => "base16-ocean.dark",
        Theme::OceanLight => "base16-ocean.light",
    };

    return (TS.themes[theme].clone(), PS.clone());
}

fn compile_sass(sass: &str) -> String {
    String::from_utf8(
        rsass::compile_scss(
            sass.as_bytes(),
            rsass::output::Format {
                style: rsass::output::Style::Compressed,
                precision: 5,
            },
        )
        .unwrap(),
    )
    .unwrap()
}
