mod errors;

use actix_web::{get, App, HttpServer};
use clap::Parser;

use crate::errors::{DrahtBotError, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "GitHub token")]
    token: String,
    #[arg(
        short,
        long = "repo",
        help = "GitHub repository slug",
        default_value = "bitcoin/bitcoin"
    )]
    repos: Vec<String>,
    #[arg(long, help = "Host to listen on", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, help = "Port to listen on", default_value = "1337")]
    port: u16,
}

struct Repository {
    owner: String,
    name: String,
}

#[get("/")]
async fn index() -> &'static str {
    "Welcome to DrahtBot!"
}

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let repos = args
        .repos
        .iter()
        .map(|repo| {
            let mut split = repo.split('/');
            let owner = split
                .next()
                .ok_or(DrahtBotError::InvalidRepositorySlug(repo.to_string()))?;
            let repo = split
                .next()
                .ok_or(DrahtBotError::InvalidRepositorySlug(repo.to_string()))?;
            Ok(Repository {
                owner: owner.to_string(),
                name: repo.to_string(),
            })
        })
        .collect::<Result<Vec<Repository>>>()?;

    println!("DrahtBot will run on the following repositories:");
    for repo in &repos {
        println!(" - {}/{}", repo.owner, repo.name);
    }

    HttpServer::new(move || App::new().service(index))
        .bind(format!("{}:{}", args.host, args.port))?
        .run()
        .await
        .map_err(|e| DrahtBotError::IOError(e))
}
