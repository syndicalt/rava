use std::error::Error;

pub fn stdout_line_value<'a>(stdout: &'a str, prefix: &str) -> Result<&'a str, Box<dyn Error>> {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .ok_or_else(|| format!("missing stdout line with prefix {prefix:?}").into())
}
