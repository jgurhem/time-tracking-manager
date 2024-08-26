use std::error::Error;

pub fn split___(s: &str) -> (&str, &str) {
    let s: Vec<&str> = s.split("___").collect();
    if s.len() == 1 {
        (s[0], "")
    } else {
        (s[0], s[1])
    }
}

pub fn split_eq(s: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let s: Vec<&str> = s.split("=").collect();
    match s.len() {
        2 => Ok((s[0], s[1])),
        1 => Err(Box::from("Rename should have an = in the middle")),
        _ => Err(Box::from("Rename should have only one = in the middle")),
    }
}