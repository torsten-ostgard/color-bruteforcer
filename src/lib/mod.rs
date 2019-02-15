//! ## Introduction
//! `color_bruteforcer` is a program that given a set of base colors C<sub>B</sub> and target colors
//! C<sub>T</sub>, attempts to find the unknown overlay color C<sub>O</sub> at opacity &alpha; that,
//! when overlayed on all elements of C<sub>B</sub>, produces the corresponding colors of
//! C<sub>T</sub>. This is done by performing a bruteforce search on the entire RGB color space and
//! alpha values from 1% to 99% opacity.

extern crate clap;
extern crate palette;
extern crate promptly;
extern crate rayon;
extern crate regex;

use std::env;
use std::fmt;

use clap::{App, Arg, ArgMatches};
use palette::white_point::D65;
use palette::{Blend, IntoColor, Laba, LinSrgb, LinSrgba};
use promptly::prompt;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;

mod alpha_generator;
mod color_distance;
pub use self::alpha_generator::AlphaGenerator;

const COLOR_REGEX: &str = r"^#?(([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2}))$";

/// A color that produces the target colors when placed over the base colors.
#[derive(Debug)]
pub struct ColorResult {
    /// The overlay color that will produce the target colors when put on top of the base colors.
    ///
    /// Stored as integer values, since image editing programs like Photoshop use those instead of
    /// floating point numbers.
    pub color: LinSrgb<u8>,
    /// The alpha value of the proposed overlay color.
    ///
    /// Stored separately to make it easy to produce a value between 1 and 99 in the display
    /// formatter. A LinSrgba<u8> type, for example, would store 50% opacity as 127, leading to
    /// annoying and imprecise conversions for display purposes.
    pub alpha: u8,
    /// The mean of the color distances between the target colors and colors produced by placing
    /// this overlay color on top of the base colors.
    pub avg_distance: f32,
}

impl fmt::Display for ColorResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#{:x} at {}% opacity; average distance: {:.6}",
            self.color, self.alpha, self.avg_distance,
        )
    }
}

/// Search all of the RGB values with a certain alpha value for a matching overlay color.
pub fn search_alpha(
    base_colors: &[LinSrgba],
    target_colors: &[Laba<D65>],
    alpha: f32,
    max_distance: f32,
) -> Vec<ColorResult> {
    (0..256)
        .into_par_iter()
        .map(|red| {
            let mut overlay = LinSrgba::new(0.0, 0.0, 0.0, alpha);
            let mut result_vector: Vec<ColorResult> = Vec::new();
            for green in 0..256 {
                for blue in 0..256 {
                    overlay.red = red as f32 / 255.0;
                    overlay.green = green as f32 / 255.0;
                    overlay.blue = blue as f32 / 255.0;
                    let result = find_match(base_colors, target_colors, overlay, max_distance);
                    if result.is_some() {
                        result_vector.push(result.unwrap());
                    }
                }
            }
            result_vector
        })
        .flatten()
        .collect()
}

fn find_match(
    base_colors: &[LinSrgba],
    target_colors: &[Laba<D65>],
    overlay: LinSrgba,
    max_distance: f32,
) -> Option<ColorResult> {
    let mut distances = Vec::with_capacity(base_colors.len());

    for (base, target) in base_colors.iter().zip(target_colors.iter()) {
        let guess = overlay.over(*base);
        let guess_lab: Laba<D65> = guess.into();
        let distance = color_distance::distance(&target, &guess_lab);

        if distance > max_distance {
            return None;
        }

        distances.push(distance);
    }

    let color: LinSrgb<u8> = overlay.into_rgb().into_format();
    let alpha = (overlay.alpha * 100.0) as u8;
    let avg_distance = distances.iter().sum::<f32>() / distances.len() as f32;
    let result = ColorResult {
        color,
        alpha,
        avg_distance,
    };

    Some(result)
}

/// Get base and target colors from either command line arguments or `stdin`.
pub fn get_colors<A, B>(arg_matches: ArgMatches) -> Result<(Vec<A>, Vec<B>), String>
where
    A: From<LinSrgba>,
    B: From<LinSrgba>,
{
    let (base_colors, target_colors) =
        if arg_matches.is_present("base_colors") || arg_matches.is_present("target_colors") {
            let num_base_colors = arg_matches
                .values_of("base_colors")
                .unwrap_or_default()
                .len();
            let num_target_colors = arg_matches
                .values_of("target_colors")
                .unwrap_or_default()
                .len();

            if num_base_colors != num_target_colors {
                let err_msg = "The number of base colors and target colors must match.";
                return Err(err_msg.to_string());
            }

            let base_colors: Vec<String> = arg_matches
                .values_of("base_colors")
                .unwrap()
                .map(|color| color.to_string())
                .collect();
            let target_colors: Vec<String> = arg_matches
                .values_of("target_colors")
                .unwrap()
                .map(|color| color.to_string())
                .collect();
            (base_colors, target_colors)
        } else {
            get_colors_from_input()
        };

    let base_colors = instantiate_colors(base_colors);
    let target_colors = instantiate_colors(target_colors);
    Ok((base_colors, target_colors))
}

fn get_colors_from_input() -> (Vec<String>, Vec<String>) {
    let mut base_colors: Vec<String> = Vec::new();
    let mut target_colors: Vec<String> = Vec::new();
    let mut iterations = 0;
    let mut raw_iterations: usize = 0;
    let r = Regex::new(COLOR_REGEX).unwrap();

    println!("Press Enter when you have finished entering all colors.");

    loop {
        raw_iterations += 1;
        let color_type = if iterations % 2 == 0 {
            "base"
        } else {
            "target"
        };
        let num_samples = (iterations / 2) + 1;
        let prompt_text = format!("Enter {} color #{}", color_type, num_samples);
        let input: Option<String> = match env::var("USE_MOCK_PROMPT") {
            Ok(_val) => mock_prompt(raw_iterations),
            Err(_e) => prompt(prompt_text),
        };

        if input.is_none() {
            if base_colors.len() == target_colors.len() {
                if base_colors.is_empty() && target_colors.is_empty() {
                    continue;
                }

                break;
            } else {
                eprintln!(
                    "You have entered {} base colors but {} target colors. Please enter the \
                     same number of each.",
                    base_colors.len(),
                    target_colors.len()
                );
                continue;
            }
        }

        let color = input.unwrap();
        let captures = r.captures(color.as_str());

        match captures {
            Some(c) => {
                let destination = if iterations % 2 == 0 {
                    &mut base_colors
                } else {
                    &mut target_colors
                };
                destination.push(String::from(&c[1]));
                iterations += 1;
            }
            None => eprintln!("Invalid color format. Please enter a valid hex code."),
        }
    }

    (base_colors, target_colors)
}

fn instantiate_colors<T>(colors: Vec<String>) -> Vec<T>
where
    T: From<LinSrgba>,
{
    colors
        .iter()
        .map(|color| {
            let s = trim_color(color);
            LinSrgba::new(
                f32::from(u16::from_str_radix(&s[0..2], 16).unwrap()) / 255.0,
                f32::from(u16::from_str_radix(&s[2..4], 16).unwrap()) / 255.0,
                f32::from(u16::from_str_radix(&s[4..6], 16).unwrap()) / 255.0,
                1.0,
            )
            .into()
        })
        .collect()
}

fn trim_color(color: &str) -> String {
    (if color.len() == 7 { &color[1..] } else { color }).to_string()
}

/// Return the application definition.
pub fn get_app() -> App<'static, 'static> {
    App::new("color_bruteforcer")
        .version("1.1.0")
        .author("Torsten Ostgard")
        .about("Finds an unknown, semitransparent overlay color.")
        .arg(
            Arg::with_name("alpha_min")
                .long("alpha-min")
                .default_value("1")
                .validator(validate_alpha)
                .help("The lowest opacity value to check"),
        )
        .arg(
            Arg::with_name("alpha_max")
                .long("alpha-max")
                .default_value("99")
                .validator(validate_alpha)
                .help("The highest opacity value to check"),
        )
        .arg(
            Arg::with_name("distance")
                .short("d")
                .long("distance")
                .default_value("1.0")
                .help(concat!(
                    "The maximum distance between two colors that will let a guess be considered ",
                    "a match\nA distance below 1.0 is generally considered to be visually ",
                    "indistinguishable, while 2.1 is generally considered to be a barely ",
                    "noticeable difference",
                )),
        )
        .arg(
            Arg::with_name("results")
                .short("r")
                .long("results")
                .default_value("25")
                .help(concat!(
                    "The maximum number of results to display\nSupply zero or a negative value to ",
                    "see all results.",
                )),
        )
        .arg(
            Arg::with_name("base_colors")
                .long("base-colors")
                .value_delimiter(",")
                .requires("target_colors")
                .validator(validate_color)
                .help("Comma-separated six character hex codes for the base colors"),
        )
        .arg(
            Arg::with_name("target_colors")
                .long("target-colors")
                .value_delimiter(",")
                .requires("base_colors")
                .validator(validate_color)
                .help("Comma-separated six character hex codes for the target colors"),
        )
}

fn validate_alpha(val: String) -> Result<(), String> {
    let parse_result = val.parse::<i32>();

    if parse_result.is_err() {
        return Err(format!("Cannot parse alpha \"{}\" as an integer.", val));
    }

    let alpha = parse_result.unwrap();

    if 1 <= alpha && alpha <= 99 {
        Ok(())
    } else {
        Err("The alphas to search must be between 1 and 99.".to_string())
    }
}

fn validate_color(val: String) -> Result<(), String> {
    let r = Regex::new(COLOR_REGEX).unwrap();
    if r.is_match(val.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "The color {} is not a valid six-character hex color code.",
            val
        ))
    }
}

// Only used to supply dummy data in tests because I could not find a way to mock prompt().
fn mock_prompt(i: usize) -> Option<String> {
    let return_values = vec![
        None, // Enter hit with no colors
        Some("#FFFFFF".to_string()),
        Some("1488HH".to_string()), // Invalid value
        None,                       // Enter hit with mismatched number of colors
        Some("000000".to_string()),
        None,
    ];

    return_values[i - 1].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_result_formatter() {
        let color_result = ColorResult {
            color: LinSrgb::new(255, 20, 136),
            alpha: 50,
            avg_distance: 0.5,
        };
        let result = format!("{}", color_result);
        let expected_result = "#ff1488 at 50% opacity; average distance: 0.500000";
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_search_alpha() {
        let base_colors = instantiate_colors(vec![
            "#ffffff".to_string(),
            "#bfbfbf".to_string(),
            "#808080".to_string(),
            "#404040".to_string(),
            "#000000".to_string(),
        ]);
        let target_colors = instantiate_colors(vec![
            "#f9bbbd".to_string(),
            "#cc8e90".to_string(),
            "#a06264".to_string(),
            "#733537".to_string(),
            "#46080a".to_string(),
        ]);
        let max_distance = 0.5;
        let color: LinSrgb<u8> = LinSrgb::new(237, 28, 35);
        let result = search_alpha(&base_colors, &target_colors, 0.30, max_distance);
        assert!(result.len() > 0);
        assert!(result
            .iter()
            .any(|r| r.color == color && r.alpha == 30 && r.avg_distance <= max_distance));
    }

    #[test]
    fn test_get_colors_from_args() {
        let arg_matches = get_app().get_matches_from(vec![
            "color_bruteforcer",
            "--base-colors=#FFFFFF",
            "--target-colors=000000",
        ]);
        let expected_result = Ok((
            vec![LinSrgba::new(1.0, 1.0, 1.0, 1.0)],
            vec![LinSrgba::new(0.0, 0.0, 0.0, 1.0)],
        ));
        let result = get_colors(arg_matches);
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_get_colors_from_args_mismatched_number_of_colors() {
        let arg_matches = get_app().get_matches_from(vec![
            "color_bruteforcer",
            "--base-colors=#FFFFFF,#001488",
            "--target-colors=000000",
        ]);
        let result: Result<(Vec<LinSrgba>, Vec<LinSrgba>), String> = get_colors(arg_matches);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_colors_from_stdin() {
        let arg_matches = ArgMatches::new();
        env::set_var("USE_MOCK_PROMPT", "true");
        let result: Result<(Vec<LinSrgba>, Vec<LinSrgba>), String> = get_colors(arg_matches);
        env::remove_var("USE_MOCK_PROMPT");
        let expected_result = Ok((
            vec![LinSrgba::new(1.0, 1.0, 1.0, 1.0)],
            vec![LinSrgba::new(0.0, 0.0, 0.0, 1.0)],
        ));
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_instantiate_colors() {
        let colors = vec!["#FFFFFF".to_string(), "000000".to_string()];
        let expected_result = vec![
            LinSrgba::new(1.0, 1.0, 1.0, 1.0),
            LinSrgba::new(0.0, 0.0, 0.0, 1.0),
        ];
        assert_eq!(expected_result, instantiate_colors(colors));
    }

    #[test]
    fn test_instantiate_colors_empty() {
        let colors: Vec<String> = Vec::new();
        let expected_result: Vec<LinSrgba> = Vec::new();
        assert_eq!(expected_result, instantiate_colors(colors));
    }

    #[test]
    fn test_validate_alpha() {
        assert_eq!(Ok(()), validate_alpha("50".to_string()));
        assert!(validate_alpha("a".to_string()).is_err());
        assert!(validate_alpha("-1".to_string()).is_err());
        assert!(validate_alpha("100".to_string()).is_err());
    }

    #[test]
    fn test_validate_color() {
        assert_eq!(Ok(()), validate_color("#001488".to_string()));
        assert_eq!(Ok(()), validate_color("abcdef".to_string()));
        assert_eq!(Ok(()), validate_color("FFFFFF".to_string()));
        assert!(validate_color("#fff".to_string()).is_err());
        assert!(validate_alpha("fff".to_string()).is_err());
        assert!(validate_color("1488HH".to_string()).is_err());
    }
}
