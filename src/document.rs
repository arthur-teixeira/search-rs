use crate::Lexer;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::fs::File;

pub struct Document {
    pub file_path: String,
    pub terms: HashMap<String, u32>,
    pub term_count: u32,
}

impl Document {
    pub fn from_file(file_path: &Path) -> Result<Self, String> {
        let mut terms: HashMap<String, u32> = HashMap::new();

        let mut file_content = Vec::new();
        let mut file = File::open(file_path).expect("Should open file");

        match file.read_to_end(&mut file_content) {
            Ok(_) => (),
            Err(err) => return Err(err.to_string()),
        };

        let as_string = String::from_utf8_lossy(&file_content).to_string();
        let as_chars = as_string.chars().collect::<Vec<char>>();

        let file_lexer = Lexer::new(&as_chars);

        let mut term_count = 0;
        for term in file_lexer {
            term_count += 1;
            terms.entry(term).and_modify(|c| *c += 1).or_insert(1);
        }

        let file_path = file_path.to_str().unwrap().to_string();

        Ok(Self {
            file_path,
            terms,
            term_count,
        })
    }

    pub fn tf(&self, term: &String) -> f32 {
        let term_freq = self.terms.get(term);
        match term_freq {
            Some(freq) => (*freq as f32) / (self.term_count as f32),
            None => 0f32,
        }
    }
}
