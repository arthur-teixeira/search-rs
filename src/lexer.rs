pub struct Lexer<'a> {
    pub query: &'a [String],
}

impl <'a> Lexer<'a> {
    pub fn new(input: &'a [String]) -> Self {
        Lexer { query: input }
    }
}
