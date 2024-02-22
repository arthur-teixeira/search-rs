mod lexer;
mod params;
mod document;

use lexer::Lexer;
use params::Params;
use document::Document;

use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let params = Params::new(&args)?;

    let lexer = Lexer::new(&params.query);

    let doc = Document::from_file(params.path)?;

    let mut tf = 0f32;
    for term in lexer {
        tf += doc.tf(&term);
    }

    println!("Search weight: {tf}");

    Ok(())
}
