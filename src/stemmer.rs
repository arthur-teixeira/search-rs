use whatlang::Lang;
use rust_stemmers::{Algorithm, Stemmer};

pub fn from_lang(lang: Lang) -> Option<Stemmer> {
    match lang {
        Lang::Eng => Some(Stemmer::create(Algorithm::English)),
        Lang::Por => Some(Stemmer::create(Algorithm::Portuguese)),
        _ => None
    }
}

pub fn stem(stemmer: &Option<Stemmer>, term: &String) -> String {
    match stemmer {
        None => term.clone(),
        Some(stemmer) => stemmer.stem(&term).to_string(),
    }
}

