use std::collections::HashSet;

/// Maintain the state necessary to generate alpha values in an intelligent manner.
#[derive(Debug, Default)]
pub struct AlphaGenerator {
    alpha_min: u8,
    alpha_max: u8,
    previous_alpha: u8,
    first_hit: Option<u8>,
    guesses: Vec<u8>,
    should_check: Vec<bool>,
}

fn generate_initial_guesses(alpha_min: u8, alpha_max: u8) -> Vec<u8> {
    let num_alphas = alpha_max - alpha_min;
    let mut guesses = Vec::with_capacity(usize::from(num_alphas));
    let midpoint = num_alphas / 2 + alpha_min;
    let mut previously_generated = HashSet::new();
    for step_size in &[3u8, 1u8] {
        let mut alpha = midpoint;
        while alpha_min <= alpha && alpha <= alpha_max {
            if !previously_generated.contains(&alpha) {
                guesses.push(alpha)
            }
            previously_generated.insert(alpha);
            let diff = i16::from(alpha) - i16::from(midpoint);

            alpha = if diff <= 0 {
                midpoint + diff.abs() as u8 + step_size
            } else {
                midpoint - diff as u8
            };
        }
    }

    // Reverse so that the values are returned by pop() in the desired order.
    guesses.reverse();

    guesses
}

impl AlphaGenerator {
    /// Initialize a new alpha generator.
    pub fn new(alpha_min: u8, alpha_max: u8) -> Self {
        let mut a = AlphaGenerator::default();
        a.alpha_min = alpha_min;
        a.alpha_max = alpha_max;
        a.guesses = generate_initial_guesses(a.alpha_min, a.alpha_max);
        a.should_check = vec![false; 99];

        for i in a.alpha_min..=a.alpha_max {
            let i = usize::from(i);
            a.should_check[i - 1] = true;
        }

        a
    }

    /// Generate the next alpha value to be searched.
    ///
    /// The function first takes large steps away from the midpoint between the minimum and maximum
    /// alpha values until a match is found. It alternates between alpha values higher and lower
    /// than the midpoint. Once a match has been found, the function then generates alpha values
    /// both higher and lower than the first match until no matches are found in both directions.
    /// If somehow the large steps produce no matches, the step size changes to one and very alpha
    /// value - excluding the ones already checked - are generated, again going outward from the
    /// midpoint, alternating between values larger and smaller than the midpoint.
    pub fn next(&mut self, had_results: bool) -> Option<u8> {
        if had_results && self.first_hit.is_none() {
            self.first_hit = Some(self.previous_alpha);
            self.guesses = self.generate_guesses_after_first_hit();
        } else if !had_results && self.first_hit.is_some() {
            let (_drained, kept_values) = {
                if self.previous_alpha < self.first_hit.unwrap() {
                    // Only keep numbers above first hit
                    self.guesses
                        .iter()
                        .partition(|&&x| x < self.first_hit.unwrap())
                } else {
                    // Only keep numbers below first hit
                    self.guesses
                        .iter()
                        .partition(|&&x| x > self.first_hit.unwrap())
                }
            };

            self.guesses = kept_values;
        }

        let alpha = self.guesses.pop();

        if let Some(a) = alpha {
            self.should_check[usize::from(a - 1)] = false;
            self.previous_alpha = a;
        }

        alpha
    }

    fn generate_guesses_after_first_hit(&mut self) -> Vec<u8> {
        let mut guesses = Vec::new();
        let first_hit = self.first_hit.unwrap();
        for i in (self.alpha_min..first_hit).rev() {
            if !self.should_check[usize::from(i - 1)] {
                break;
            }

            guesses.push(i);
        }
        for i in (first_hit + 1)..=self.alpha_max {
            if !self.should_check[usize::from(i - 1)] {
                break;
            }

            guesses.push(i);
        }

        guesses.reverse();

        guesses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _get_range(alpha_generator: &mut AlphaGenerator, match_start: u8, match_end: u8) -> Vec<u8> {
        let mut had_results = false;
        let mut results = Vec::new();
        loop {
            let alpha = alpha_generator.next(had_results);

            match alpha {
                Some(a) => {
                    had_results = match_start <= a && a <= match_end;
                    results.push(a)
                }
                None => break,
            }
        }

        results
    }

    #[test]
    fn test_full_range_no_results_found() {
        let mut alpha_generator = AlphaGenerator::new(1, 99);
        let results = _get_range(&mut alpha_generator, 0, 0);
        let expected_result = vec![
            50, 53, 47, 56, 44, 59, 41, 62, 38, 65, 35, 68, 32, 71, 29, 74, 26, 77, 23, 80, 20, 83,
            17, 86, 14, 89, 11, 92, 8, 95, 5, 98, 2, 51, 49, 52, 48, 54, 46, 55, 45, 57, 43, 58,
            42, 60, 40, 61, 39, 63, 37, 64, 36, 66, 34, 67, 33, 69, 31, 70, 30, 72, 28, 73, 27, 75,
            25, 76, 24, 78, 22, 79, 21, 81, 19, 82, 18, 84, 16, 85, 15, 87, 13, 88, 12, 90, 10, 91,
            9, 93, 7, 94, 6, 96, 4, 97, 3, 99, 1,
        ];
        assert_eq!(expected_result, results);
    }

    #[test]
    fn test_short_range_odd_diff_no_results_found() {
        let mut alpha_generator = AlphaGenerator::new(75, 86);
        let results = _get_range(&mut alpha_generator, 0, 0);
        let expected_result = vec![80, 83, 77, 86, 81, 79, 82, 78, 84, 76, 85, 75];
        assert_eq!(expected_result, results);
    }

    #[test]
    fn test_short_range_even_diff_no_results_found() {
        let mut alpha_generator = AlphaGenerator::new(75, 85);
        let results = _get_range(&mut alpha_generator, 0, 0);
        let expected_result = vec![80, 83, 77, 81, 79, 82, 78, 84, 76, 85, 75];
        assert_eq!(expected_result, results);
    }

    #[test]
    fn test_full_range_results_found() {
        let mut alpha_generator = AlphaGenerator::new(1, 99);
        let results = _get_range(&mut alpha_generator, 73, 77);
        let expected_result = vec![
            50, 53, 47, 56, 44, 59, 41, 62, 38, 65, 35, 68, 32, 71, 29, 74, 73, 72, 75, 76, 77, 78,
        ];
        assert_eq!(expected_result, results);
    }

    #[test]
    fn test_matches_exist_outside_of_alpha_range() {
        let mut alpha_generator = AlphaGenerator::new(19, 31);
        let results = _get_range(&mut alpha_generator, 17, 21);
        let expected_result = vec![25, 28, 22, 31, 19, 20, 21];
        // The results should be bounded by the provided min and max alphas, so even though 17 and
        // 18 are valid matches, they should not appear in the results.
        assert_eq!(expected_result, results);
    }
}
