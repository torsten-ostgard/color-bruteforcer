#[cfg(test)]
mod tests {
    const BASE_NUM_LINES: usize = 3;
    use std::process::{Command, Stdio};

    #[test]
    fn test_no_results() {
        let output = Command::new("./target/debug/color_bruteforcer")
            .arg("--alpha-min=1")
            .arg("--alpha-max=1")
            .arg("--base-colors=#ffffff,#bfbfbf,#808080,#404040,#000000")
            .arg("--target-colors=#fabbbd,#cd8e91,#a16264,#743538,#47080b")
            .output()
            .expect("failed to start color_bruteforcer");
        match output.status.code() {
            Some(code) => assert_eq!(1, code),
            None => assert!(false, "Process terminated by signal"),
        }
        let output_text = String::from_utf8_lossy(&output.stdout);
        let num_lines = output_text.lines().collect::<Vec<&str>>().len();
        assert_eq!(BASE_NUM_LINES, num_lines);
    }

    #[test]
    fn test_has_results() {
        let output = Command::new("./target/debug/color_bruteforcer")
            .arg("--alpha-min=30")
            .arg("--alpha-max=30")
            .arg("--base-colors=#ffffff,#bfbfbf,#808080,#404040,#000000")
            .arg("--target-colors=#fabbbd,#cd8e91,#a16264,#743538,#47080b")
            .arg("--results=5")
            .output()
            .expect("failed to start color_bruteforcer");
        assert!(output.status.success());
        let output_text = String::from_utf8_lossy(&output.stdout);
        let lines = output_text.lines().collect::<Vec<&str>>();
        let parse_as_f32 = |x: &str| x.split(" ").last().unwrap().parse().unwrap();
        let first_dist: f32 = parse_as_f32(lines.get(3).unwrap());
        let last_dist: f32 = parse_as_f32(lines.last().unwrap());
        assert!(first_dist < last_dist);
        assert_eq!(BASE_NUM_LINES + 5, lines.len());
    }

    #[test]
    fn test_alpha_conflict() {
        let status = Command::new("./target/debug/color_bruteforcer")
            .arg("--alpha-min=80")
            .arg("--alpha-max=70")
            .stderr(Stdio::null())
            .status()
            .expect("failed to start color_bruteforcer");
        match status.code() {
            Some(code) => assert_eq!(1, code),
            None => assert!(false, "Process terminated by signal"),
        }
    }

    #[test]
    fn test_get_colors_error() {
        let status = Command::new("./target/debug/color_bruteforcer")
            .arg("--base-colors=#FFFFFF,#001488")
            .arg("--target-colors=000000")
            .stderr(Stdio::null())
            .status()
            .expect("failed to start color_bruteforcer");
        match status.code() {
            Some(code) => assert_eq!(1, code),
            None => assert!(false, "Process terminated by signal"),
        }
    }
}
