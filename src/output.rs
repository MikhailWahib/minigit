fn paint(text: &str, code: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

pub fn green(text: &str) -> String {
    paint(text, "32")
}

pub fn red(text: &str) -> String {
    paint(text, "31")
}

pub fn yellow(text: &str) -> String {
    paint(text, "33")
}
