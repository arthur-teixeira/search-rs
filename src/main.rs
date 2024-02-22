mod lexer;
mod params;
mod document;
mod corpus;

use lexer::Lexer;
use params::Params;
use document::Document;
use corpus::Corpus;

use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let params = Params::new(&args)?;

    let terms = Lexer::new(&params.query);

    let corpus = match Corpus::from_folder(&params.path) {
        Ok(corpus) => corpus,
        Err(e) => return Err(e.to_string()),
    };

    let classification = corpus.classify(terms);

    for (file, score) in classification {
        println!("{file}: {score}");
    }

    Ok(())
}
