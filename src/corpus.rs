use crate::lexer::Lexer;
use crate::Document;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub struct Corpus {
    pub docs: Vec<Document>,
}

impl Corpus {
    pub fn from_folder(path: &Path) -> Result<Self, Error> {
        if !path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "The provided path should be a directory",
            ));
        }

        let dir = fs::read_dir(path)?;
        let mut docs: Vec<Document> = Vec::new();

        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            println!("visiting {0:?}", path.as_os_str());

            if path.is_dir() {
                continue; // TODO: visit nested directories
            }

            let doc = match Document::from_file(&path) {
                Ok(doc) => doc,
                Err(msg) => return Err(Error::new(ErrorKind::InvalidInput, msg)),
            };

            docs.push(doc);
        }

        Ok(Corpus { docs })
    }

    fn idf(&self, term: &String) -> f32 {
        let mut score = 0f32;

        for doc in &self.docs {
            if doc.tf(term) > 0f32 {
                score += 1f32;
            }
        }

        return (self.docs.len() as f32 / score).log10();
    }

    pub fn classify(&self, terms: Lexer) -> Vec<(String, f32)> {
        let mut result = Vec::new();
        let terms: Vec<String> = terms.collect();

        for doc in &self.docs {
            let mut score = 0f32;
            for term in &terms {
                score += doc.tf(&term) * self.idf(&term);
            }

            result.push((doc.file_name.clone(), score));
        }

        result
    }
}
