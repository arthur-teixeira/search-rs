use crate::lexer::Lexer;
use crate::stemmer::Stem;
use crate::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use whatlang::Lang;

type DocFreq = HashMap<String, f32>;

#[derive(Serialize, Deserialize)]
pub struct Corpus {
    pub docs: HashMap<PathBuf, Document>,
    doc_freq: DocFreq,
    language: Lang,
}

fn visit_files(initial_path: &Path) -> Result<Vec<PathBuf>, Error> {
    if !initial_path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "The provided path should be a directory",
        ));
    }

    let mut dirs_to_visit = Vec::new();
    dirs_to_visit.push(initial_path.to_path_buf());

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

            let dot_file = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("."))
                .unwrap_or(false);

            if dot_file {
                continue;
            }

            files_to_visit.push(path);
        }
    }

    Ok(files_to_visit)
}

impl Corpus {
    pub fn from_folder(path: &Path) -> Result<Self, Error> {
        let cache_file = path.join(".cache.json");

        if cache_file.exists() {
            let ret: Self = serde_json::from_reader(fs::File::open(cache_file)?)?;
            return Ok(ret);
        }

        let num_threads = thread::available_parallelism()?;

        let mut handles = Vec::with_capacity(num_threads.into());
        let freq_arc = Arc::new(Mutex::new(DocFreq::new()));
        let files_to_visit = visit_files(path)?;
        let n = files_to_visit.len();

        for i in 0..num_threads.into() {
            let freq_lock = freq_arc.clone();
            let files_to_visit = files_to_visit.clone();

            let handle = thread::spawn(move || {
                let mut docs: HashMap<PathBuf, Document> = HashMap::new();
                let files = files_to_visit[(i * n) / 4..((i + 1) * n) / 4].to_vec();

                for path in files {
                    let doc = match Document::from_file(&path) {
                        Ok(doc) => doc,
                        Err(e) => {
                            eprintln!("ERROR: {e}. Skipping file");
                            continue;
                        }
                    };

                    let mut freq_table = freq_lock.lock().unwrap();

                    for k in doc.terms.keys() {
                        freq_table
                            .entry(k.to_string())
                            .and_modify(|c| *c += 1f32)
                            .or_insert(1f32);
                    }

                    docs.insert(path, doc);
                }

                docs
            });

            handles.push(handle);
        }

        let mut docs = HashMap::new();
        for handle in handles {
            let result = handle.join().unwrap();
            docs.extend(result);
        }

        let val = freq_arc.lock().unwrap().clone();
        let language = docs.values().next().unwrap().language; // Assumes all documents are in the same language

        let ret = Corpus {
            docs,
            doc_freq: val,
            language,
        };

        serde_json::to_writer(fs::File::create(cache_file)?, &ret)?;

        Ok(ret)
    }

    fn idf(&self, term: &String) -> f32 {
        let df = self.doc_freq.get(term).unwrap_or(&1f32);
        (self.docs.len() as f32 / df).log10()
    }

    pub fn classify(&self, terms: Lexer) -> Vec<(String, f32)> {
        let mut result = Vec::new();
        let terms: Vec<String> = terms.collect();

        let stemmer = Stem::from_lang(self.language);

        eprintln!("Searching in {0} documents", self.docs.len());
        for (_, doc) in &self.docs {
            let mut score = 0f32;

            for term in &terms {
                let stem = stemmer.stem(&term);
                let tf = doc.tf(&stem);
                let idf = self.idf(&stem);
                score += tf * idf;
            }

            result.push((doc.file_path.clone(), score));
        }
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        return result
            .iter()
            .filter(|(_, score)| score > &0f32)
            .cloned()
            .collect();
    }
}
