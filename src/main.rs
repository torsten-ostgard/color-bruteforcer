//! ## Introduction
//! `color_bruteforcer` is a program that given a set of base colors C<sub>B</sub> and target colors
//! C<sub>T</sub>, attempts to find the unknown overlay color C<sub>O</sub> at opacity &alpha; that,
//! when overlaid on all elements of C<sub>B</sub>, produces the corresponding colors of
//! C<sub>T</sub>. This is done by performing a bruteforce search on the entire RGB color space and
//! alpha values from 1% to 99% opacity.
#![deny(missing_docs, nonstandard_style, unsafe_code, rust_2018_idioms)]
use std::process::exit;

use clap::value_t_or_exit;
use pbr::ProgressBar;

mod lib;
use lib::{get_app, get_colors, search_alpha, AlphaGenerator, ColorResult};

fn main() {
    let arg_matches = get_app().get_matches();
    let alpha_min = value_t_or_exit!(arg_matches, "alpha_min", u8);
    let alpha_max = value_t_or_exit!(arg_matches, "alpha_max", u8);
    let max_distance = value_t_or_exit!(arg_matches, "distance", f32);
    let max_results = value_t_or_exit!(arg_matches, "results", isize);

    if alpha_min > alpha_max {
        eprintln!("alpha-min must be less than or equal to alpha-max.");
        exit(1);
    }

    let (base_colors, target_colors) = get_colors(arg_matches).unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });

    let mut color_results = Vec::new();
    let num_alphas = u64::from((alpha_max + 1) - alpha_min);
    let mut pb = ProgressBar::new(num_alphas);
    pb.show_counter = false;
    pb.message("Found 0 possible colors ");
    let mut alpha_generator = AlphaGenerator::new(alpha_min, alpha_max);
    let mut previous_alpha_had_results = false;

    while let Some(alpha_int) = alpha_generator.next(previous_alpha_had_results) {
        let alpha = f32::from(alpha_int) / 100.0;
        let mut alpha_results = search_alpha(&base_colors, &target_colors, alpha, max_distance);
        previous_alpha_had_results = !alpha_results.is_empty();
        color_results.append(&mut alpha_results);
        pb.inc();
        let message = format!("Found {} possible colors ", color_results.len());
        pb.message(message.as_str());
    }

    pb.finish();

    if color_results.is_empty() {
        println!("\n\nNo results found.");
        exit(1);
    }

    // avg_distance will always be a valid float, so unwrap() can be done safely.
    color_results.sort_by(|a, b| a.avg_distance.partial_cmp(&b.avg_distance).unwrap());

    let prefix = get_prefix(max_results, &mut color_results);
    println!("\n\n{}, starting from the most similar color:", prefix);

    for result in color_results {
        println!("{}", result);
    }
}

fn get_prefix(max_results: isize, color_results: &mut Vec<ColorResult>) -> String {
    if max_results < 1 || color_results.len() as isize <= max_results {
        "All results".to_string()
    } else {
        color_results.truncate(max_results as usize);
        if max_results > 1 {
            format!("Top {} results", max_results)
        } else {
            "Top result".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::LinSrgb;

    #[test]
    fn test_get_prefix_one_result() {
        let mut color_results: Vec<ColorResult> = (1..100)
            .into_iter()
            .map(|i| ColorResult {
                color: LinSrgb::new(0, 0, 0),
                alpha: i,
                avg_distance: 0.0,
            })
            .collect();
        assert_eq!(99, color_results.len());
        let result = get_prefix(1, &mut color_results);
        assert_eq!("Top result".to_string(), result);
        assert_eq!(1, color_results.len());
    }

    #[test]
    fn test_get_prefix_n_results() {
        let mut color_results: Vec<ColorResult> = (1..100)
            .into_iter()
            .map(|i| ColorResult {
                color: LinSrgb::new(0, 0, 0),
                alpha: i,
                avg_distance: 0.0,
            })
            .collect();
        assert_eq!(99, color_results.len());
        let result = get_prefix(5, &mut color_results);
        assert_eq!("Top 5 results".to_string(), result);
        assert_eq!(5, color_results.len());
    }

    #[test]
    fn test_get_prefix_unlimited_results() {
        let mut color_results: Vec<ColorResult> = (1..100)
            .into_iter()
            .map(|i| ColorResult {
                color: LinSrgb::new(0, 0, 0),
                alpha: i,
                avg_distance: 0.0,
            })
            .collect();
        assert_eq!(99, color_results.len());
        let result = get_prefix(0, &mut color_results);
        assert_eq!("All results".to_string(), result);
        assert_eq!(99, color_results.len());
    }

    #[test]
    fn test_get_prefix_fewer_than_limit() {
        let mut color_results: Vec<ColorResult> = (1..100)
            .into_iter()
            .map(|i| ColorResult {
                color: LinSrgb::new(0, 0, 0),
                alpha: i,
                avg_distance: 0.0,
            })
            .collect();
        assert_eq!(99, color_results.len());
        let result = get_prefix(2000, &mut color_results);
        assert_eq!("All results".to_string(), result);
        assert_eq!(99, color_results.len());
    }
}
