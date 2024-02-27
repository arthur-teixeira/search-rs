use crate::lexer::Lexer;
use crate::Document;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex, self};

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

        let mut dirs_to_visit = Vec::new();
        dirs_to_visit.push(path.to_path_buf());

        let mut files_to_visit = Vec::new();

        while dirs_to_visit.len() > 0 {
            let dir_to_visit = dirs_to_visit.pop().unwrap();
            let dir = fs::read_dir(dir_to_visit)?;

            for entry in dir {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_visit.push(path);
                    continue;
                }

                files_to_visit.push(path);
            }
        }

        let n = files_to_visit.len();

        let mut handles = Vec::with_capacity(4);
        let freq_arc = Arc::new(Mutex::new(DocFreq::new()));

        let num_threads = thread::available_parallelism()?;

        for i in 0..num_threads.into() {
            let files_to_visit = files_to_visit.clone();
            let freq_lock = freq_arc.clone();

            let handle = thread::spawn(move || {
                let mut docs: Vec<Document> = Vec::new();
                let files = files_to_visit[(i * n) / 4..((i + 1) * n) / 4].to_vec();

                for path in files {
                    let doc = match Document::from_file(&path) {
                        Ok(doc) => doc,
                        Err(msg) => return Err(Error::new(ErrorKind::InvalidInput, msg)),
                    };

                    let mut freq_table = freq_lock.lock().unwrap();

                    for k in doc.terms.keys() {
                        freq_table
                            .entry(k.to_string())
                            .and_modify(|c| *c += 1f32)
                            .or_insert(1f32);
                    }
                    
                    docs.push(doc);
                }

                Ok(docs)
            });

            handles.push(handle);
        }

        let mut docs = Vec::new();
        for handle in handles {
            let result = handle.join().unwrap()?;
            docs.extend(result);
        }

        let val = freq_arc.lock().unwrap().clone();

        Ok(Corpus { docs, doc_freq: val })
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
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        return result;
    }
}
