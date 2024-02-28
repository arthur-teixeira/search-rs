use crate::stemmer::Stem;
use crate::Lexer;
use docx_rs::{self, DocumentChild};
use poppler;
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use whatlang::{detect, Info, Lang};

pub struct Document {
    pub file_path: String,
    pub terms: HashMap<String, u32>,
    pub term_count: u32,
    pub language: Lang,
}

fn read_raw_text(file_path: &Path) -> Result<Vec<char>, String> {
    let mut file_content = Vec::new();
    let mut file = File::open(file_path).expect("Should open file");

    match file.read_to_end(&mut file_content) {
        Ok(_) => (),
        Err(err) => return Err(err.to_string()),
    };

    let as_string = String::from_utf8_lossy(&file_content).to_string();

    Ok(as_string.chars().collect::<Vec<char>>())
}

fn read_pdf(file_path: &Path) -> Result<Vec<char>, String> {
    let mut content = Vec::new();
    let _ = File::open(file_path).unwrap().read_to_end(&mut content);

    let file = match poppler::Document::from_data(&content, None) {
        Ok(doc) => doc,
        Err(e) => return Err(e.to_string()),
    };

    let n = file.n_pages();

    let mut full_text: Vec<char> = Vec::new();

    for i in 0..n {
        let page = file.page(i).unwrap();
        let text = page.text().unwrap();
        let chs: Vec<char> = text.to_string().chars().collect();
        full_text.push(' ');
        full_text.extend(chs);
    }

    Ok(full_text)
}

fn read_docx(file_path: &Path) -> Result<Vec<char>, String> {
    let mut content = Vec::new();
    let _ = File::open(file_path).unwrap().read_to_end(&mut content);

    let file = match docx_rs::read_docx(&content) {
        Ok(doc) => doc,
        Err(e) => return Err(e.to_string()),
    };

    let children = file.document.children;
    let paragraphs = children.iter().filter_map(|c| match c {
        DocumentChild::Paragraph(content) => Some(content),
        _ => None,
    });

    let text = paragraphs
        .map(|p| p.raw_text())
        .collect::<Vec<String>>()
        .join(" ")
        .chars()
        .collect();

    Ok(text)
}

fn read_text(file_path: &Path) -> Result<Vec<char>, String> {
    let extension = file_path
        .extension()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap();

    match extension {
        "txt" | "" => read_raw_text(file_path),
        "pdf" => read_pdf(file_path),
        "docx" => read_docx(file_path),
        _ => {
            eprintln!("Warning: Unexpected file type {extension}, reading as raw text");
            read_raw_text(file_path)
        }
    }
}

fn stop_words(language: Lang) -> Result<HashSet<String>, Error> {
    let root = env::current_dir()?;

    let file = match language {
        Lang::Eng => Some("./stopwords/english"),
        Lang::Spa => Some("./stopwords/spanish"),
        Lang::Por => Some("./stopwords/portuguese"),
        _ => None,
    }
    .expect("Should not accept other languages");

    let path = root.join(PathBuf::from(file));

    let mut file = File::open(path).expect("Should have file");

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    Ok(content.lines().map(|s| s.to_string()).collect())
}

fn accepted_language(language: whatlang::Lang) -> bool {
    match language {
        Lang::Eng | Lang::Spa | Lang::Por => true,
        _ => false,
    }
}

impl Document {
    pub fn from_file(file_path: &Path) -> Result<Self, String> {
        let mut terms: HashMap<String, u32> = HashMap::new();

        let as_chars = read_text(file_path)?;
        let as_str = as_chars.iter().collect::<String>();

        let doc_lang = match detect(&as_str) {
            Some(info) => {
                if !info.is_reliable() {
                    Lang::Eng
                } else {
                    info.lang()
                }
            }
            None => Lang::Eng,
        };

        if !accepted_language(doc_lang) {
            eprintln!("Language {doc_lang} for file {file_path:?} not supported");
            return Err(format!(""));
        }

        let stop_words = match stop_words(doc_lang) {
            Ok(s) => s,
            Err(e) => return Err(e.to_string()),
        };

        let stemmer = Stem::from_lang(doc_lang);

        let file_lexer = Lexer::new(&as_chars);
        let valid_terms = file_lexer.filter(|t| !stop_words.contains(t));

        let mut term_count = 0;
        for term in valid_terms {
            term_count += 1;
            let stem = stemmer.stem(&term);

            terms.entry(stem).and_modify(|c| *c += 1).or_insert(1);
        }

        let file_path = file_path.to_str().unwrap().to_string();

        Ok(Self {
            file_path,
            terms,
            term_count,
            language: doc_lang,
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
