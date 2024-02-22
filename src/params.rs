use std::path::Path;

pub struct Params {
    pub path: Box<Path>,
    pub query: Vec<char>,
}

impl Params {
    pub fn new(args: &Vec<String>) -> Result<Self, String> {
        if args.len() < 3 {
            return Err(format!("Usage: {} [filename] [query]", args[0]));
        }

        let path = args[1].clone();
        let path: Box<Path> = Path::new(&path).into();

        let query = args[2..].join(" ").chars().collect();

        Ok(Params { path, query })
    }
}
