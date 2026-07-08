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
