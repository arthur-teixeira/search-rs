pub struct Params<'a> {
    pub path: String,
    pub query: &'a [String],
}

impl<'a> Params<'a> {
    pub fn new(args: &'a Vec<String>) -> Result<Self, String> {
        if args.len() < 3 {
            return Err(format!("Usage: {} [filename] [query]", args[0]));
        }
        let path = args[1].clone();

        Ok(Params {
            path,
            query: &args[2..],
        })
    }
}
