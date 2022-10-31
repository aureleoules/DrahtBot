use thiserror::Error;

pub type Result<T> = std::result::Result<T, DrahtBotError>;

#[derive(Error, Debug)]
pub enum DrahtBotError {
    #[error("Invalid repository slug: {0}")]
    InvalidRepositorySlug(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("GitHub Error {0}")]
    GitHubError(#[from] octocrab::Error),
}
