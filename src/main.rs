mod corpus;
mod document;
mod lexer;
mod stemmer;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use corpus::Corpus;
use document::Document;
use lexer::Lexer;
use serde::Deserialize;
use std::env;
use std::path::Path;

fn init_state() -> std::io::Result<Corpus> {
    let args: Vec<String> = env::args().collect();
    let program = args.iter().next().expect("Program");

    let path = match args.get(1) {
        Some(path) => path,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Usage: {} [folder]", program),
            ))
        }
    };

    let corpus = Corpus::from_folder(Path::new(path))?;

    eprintln!("Finished indexing");

    Ok(corpus)
}

#[derive(Deserialize)]
struct Search {
    query: String,
}

#[get("/search")]
async fn query(data: web::Data<Corpus>, search: web::Query<Search>) -> impl Responder {
    let chars = search.query.chars().collect::<Vec<char>>();
    let lexer = Lexer::new(&chars);

    let result = data.classify(lexer);

    HttpResponse::Ok().json(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        eprintln!("Starting server");
        let corpus = match init_state() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        App::new().app_data(web::Data::new(corpus)).service(query)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
