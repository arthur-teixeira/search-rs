mod lexer;
mod params;

use lexer::Lexer;
use params::Params;
use std::{collections::HashMap, env, fs};

struct Document {
    pub terms: HashMap<String, u32>,
    pub term_count: u32,
}

impl Document {
    pub fn from_file(file_path: String) -> Result<Self, String> {
        let mut terms: HashMap<String, u32> = HashMap::new();

        let file = match fs::read_to_string(file_path) {
            Ok(bs) => bs,
            Err(err) => return Err(err.to_string()),
        };

        let fs: Vec<char> = file.chars().collect();
        let file_lexer = Lexer::new(&fs);

        let mut term_count = 0;
        for term in file_lexer {
            term_count += 1;
            terms.entry(term).and_modify(|c| *c += 1).or_insert(1);
        }

        Ok(Self { terms, term_count })
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let params = Params::new(&args)?;

    let mut lexer = Lexer::new(&params.query);

    let search_term = lexer.next_token().unwrap();
    let doc = Document::from_file(params.path)?;

    let tf = get_tf(&search_term, &doc);

    println!("tf of term: {tf}");

    Ok(())
}

fn get_tf(term: &String, doc: &Document) -> f32 {
    let term_freq = doc.terms.get(term);
    match term_freq {
        Some(freq) => (*freq as f32) / (doc.term_count as f32),
        None => 0f32,
    }
}
