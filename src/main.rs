mod lexer;
mod params;

use lexer::Lexer;
use params::Params;
use std::{collections::HashMap, env, fs};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let params = Params::new(&args)?;
    let _lexer = Lexer::new(&params.query);

    let file = match fs::read_to_string(params.path) {
        Ok(bs) => bs,
        Err(err) => return Err(err.to_string()),
    };

    let mut term_map: HashMap<&str, usize> = HashMap::new();
    let terms: Vec<&str> = file.split_whitespace().collect();

    for t in terms {
        term_map.entry(t).and_modify(|c| *c += 1).or_insert(1);
    }

    for (k, v) in term_map.into_iter() {
        println!("{k} freq: {v}");
    }

    return Ok(());
}
