#[derive(Debug)]
pub struct ParseError {
    str: String,
}

impl ParseError {
    pub fn new(str: &str) -> Self {
        ParseError {
            str: str.to_string(),
        }
    }

    pub fn print(&self) {
        println!("{}", self.str);
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseError {}
