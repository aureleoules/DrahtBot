mod errors;
mod features;

use std::str::FromStr;

use actix_web::{get, post, web, App, HttpRequest, HttpServer, Responder};
use clap::Parser;
use features::Feature;
use octocrab::Octocrab;
use strum::{Display, EnumString};

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

#[derive(Debug, Clone)]
struct Repository {
    owner: String,
    name: String,
}

#[derive(Debug, Display, EnumString, PartialEq, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum GitHubEvent {
    Create,
    IssueComment,
    Ping,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    Push,

    Unknown,
}

#[get("/")]
async fn index() -> &'static str {
    "Welcome to DrahtBot!"
}

#[derive(Debug, Clone)]
pub struct Context {
    octocrab: Octocrab,
}

#[post("/postreceive")]
async fn postreceive_handler(
    ctx: web::Data<Context>,
    req: HttpRequest,
    data: web::Json<serde_json::Value>,
) -> impl Responder {
    let event_str = req
        .headers()
        .get("X-GitHub-Event")
        .unwrap()
        .to_str()
        .unwrap();
    let event = GitHubEvent::from_str(event_str).unwrap_or(GitHubEvent::Unknown);

    emit_event(&ctx, event, data).await.unwrap();

    "OK"
}

fn features() -> Vec<Box<dyn Feature>> {
    vec![]
}

async fn emit_event(
    ctx: &Context,
    event: GitHubEvent,
    data: web::Json<serde_json::Value>,
) -> Result<()> {
    for feature in features() {
        if feature.meta().events().contains(&event) {
            feature.handle(ctx, event, &data).await?;
        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let octocrab = octocrab::Octocrab::builder()
        .personal_token(args.token)
        .build()
        .map_err(|e| DrahtBotError::GitHubError(e))?;

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

    println!("");

    println!("DrahtBot will will run the following features:");
    for feature in features() {
        println!(" - {}", feature.meta().name());
        println!("   {}", feature.meta().description());
    }

    let context = Context { octocrab };

    HttpServer::new(move || {
        App::new()
            .app_data(context.clone())
            .service(index)
            .service(postreceive_handler)
    })
    .bind(format!("{}:{}", args.host, args.port))?
    .run()
    .await
    .map_err(|e| DrahtBotError::IOError(e))
}
