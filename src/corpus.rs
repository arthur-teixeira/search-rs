use crate::lexer::Lexer;
use crate::stemmer::Stem;
use crate::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use whatlang::Lang;

type DocFreq = HashMap<String, usize>;

#[derive(Serialize, Deserialize)]
pub struct Corpus {
    pub docs: HashMap<PathBuf, Document>,
    doc_freq: DocFreq,
    language: Lang,
}

fn visit_files(initial_path: &Path, threads: &Vec<Sender<PathBuf>>) -> Result<(), Error> {
    if !initial_path.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "The provided path should be a directory",
        ));
    }

    let mut dirs_to_visit = Vec::new();
    dirs_to_visit.push(initial_path.to_path_buf());

    let mut i = 0;
    let num_threads = threads.len();

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

            threads[i % num_threads].send(path).unwrap();
            i += 1;
        }
    }

    Ok(())
}

fn spawn_threads(doc_snd: Sender<(PathBuf, Document)>) -> Vec<Sender<PathBuf>> {
    let num_threads: usize = thread::available_parallelism().unwrap().into();

    let mut worker_thread_handles = Vec::with_capacity(num_threads);

    for _ in 0..num_threads.into() {
        let sender = doc_snd.clone();

        let (path_snd, path_recv) = channel::<PathBuf>();
        worker_thread_handles.push(path_snd);

        thread::spawn(move || {
            for path in path_recv {
                let doc = match Document::from_file(&path) {
                    Ok(doc) => doc,
                    Err(e) => {
                        eprintln!("ERROR: {e}. Skipping file");
                        continue;
                    }
                };

                sender.send((path, doc)).unwrap();
            }

            drop(sender);
        });
    }

    drop(doc_snd);

    worker_thread_handles
}

impl Corpus {
    pub fn from_folder(path: &Path) -> Result<Self, Error> {
        let cache_file = path.join(".cache.json");

        if cache_file.exists() {
            let ret: Self = serde_json::from_reader(fs::File::open(cache_file)?)?;
            return Ok(ret);
        }

        let mut doc_freq = DocFreq::new();
        let (doc_snd, doc_recv) = channel::<(PathBuf, Document)>();

        let worker_thread_handles = spawn_threads(doc_snd);

        visit_files(path, &worker_thread_handles)?;

        for handle in worker_thread_handles {
            drop(handle);
        }

        let mut docs = HashMap::new();
        for (path, doc) in doc_recv {
            for term in doc.terms.keys() {
                doc_freq.entry(term.to_string()).and_modify(|v| *v += 1).or_insert(1);
            }
            docs.insert(path, doc);
        }

        let language = docs.iter().next().unwrap().1.language;

        let ret = Corpus {
            docs,
            doc_freq,
            language,
        };

        serde_json::to_writer(fs::File::create(cache_file)?, &ret)?;

        Ok(ret)
    }

    fn idf(&self, term: &String) -> f32 {
        let df = self.doc_freq.get(term).unwrap_or(&1);
        (self.docs.len() as f32 / (*df as f32)).log10()
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
