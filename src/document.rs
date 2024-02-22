use crate::Lexer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Document {
    pub file_name: String,
    pub terms: HashMap<String, u32>,
    pub term_count: u32,
}

impl Document {
    pub fn from_file(file_path: &Path) -> Result<Self, String> {
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

        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

        Ok(Self {
            file_name,
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
