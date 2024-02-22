pub struct Params {
    pub path: String,
    pub query: Vec<char>
}

impl Params {
    pub fn new(args: &Vec<String>) -> Result<Self, String> {
        if args.len() < 3 {
            return Err(format!("Usage: {} [filename] [query]", args[0]));
        }
        let path = args[1].clone();

        let query = args[2..].join(" ").chars().collect();

        Ok(Params {
            path,
            query,
        })
    }
}
