use crate::lexer::Lexer;
use crate::Document;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

type DocFreq = HashMap<String, f32>;

pub struct Corpus {
    pub docs: Vec<Document>,
    doc_freq: DocFreq,
}

impl Corpus {
    pub fn from_folder(path: &Path) -> Result<Self, Error> {
        if !path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "The provided path should be a directory",
            ));
        }

        let mut doc_freq = DocFreq::new();

        let mut dirs_to_visit = Vec::new();
        dirs_to_visit.push(path.to_path_buf());

        let mut docs: Vec<Document> = Vec::new();

        while dirs_to_visit.len() > 0 {
            let dir_to_visit = dirs_to_visit.pop().unwrap();
            let dir = fs::read_dir(dir_to_visit)?;

            for entry in dir {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_visit.push(path.to_path_buf());
                    continue;
                }

                let doc = match Document::from_file(&path) {
                    Ok(doc) => doc,
                    Err(msg) => return Err(Error::new(ErrorKind::InvalidInput, msg)),
                };

                for k in doc.terms.keys() {
                    doc_freq
                        .entry(k.to_string())
                        .and_modify(|c| *c += 1f32)
                        .or_insert(1f32);
                }

                docs.push(doc);
            }
        }

        Ok(Corpus { docs, doc_freq })
    }

    fn idf(&self, term: &String) -> f32 {
        let df = self.doc_freq.get(term).unwrap_or(&1f32);
        return (self.docs.len() as f32 / df).log10();
    }

    pub fn classify(&self, terms: Lexer) -> Vec<(String, f32)> {
        let mut result = Vec::new();
        let terms: Vec<String> = terms.collect();

        for doc in &self.docs {
            let mut score = 0f32;
            for term in &terms {
                score += doc.tf(&term) * self.idf(&term);
            }

            result.push((doc.file_path.clone(), score));
        }

        result
    }
}
