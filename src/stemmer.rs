use rust_stemmers::{Algorithm, Stemmer};
use whatlang::Lang;

pub struct Stem {
    stemmer: Option<Stemmer>,
}

impl Stem {
    pub fn from_lang(lang: Lang) -> Self {
        let stemmer = match lang {
            Lang::Eng => Some(Stemmer::create(Algorithm::English)),
            Lang::Por => Some(Stemmer::create(Algorithm::Portuguese)),
            Lang::Spa => Some(Stemmer::create(Algorithm::Spanish)),
            _ => None,
        };

        Stem { stemmer }
    }

    pub fn stem(&self, term: &String) -> String {
        match &self.stemmer {
            None => term.clone(),
            Some(stemmer) => stemmer.stem(&term).to_string(),
        }
    }
}

