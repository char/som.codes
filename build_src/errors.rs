use std::error::Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SiteGenerationError {
    #[error("There was an issue regarding I/O: {0}")]
    IOError(std::io::Error),

    #[error("There was a {language} processing issue: {error_message}.{}", .cause.as_ref().map(|cause| format!("\nCause: {}.", cause)).unwrap_or_else(|| "".to_string()))]
    LanguageError {
        language: String,
        error_message: String,
        cause: Option<String>,
    },
}

impl From<std::io::Error> for SiteGenerationError {
    fn from(error: std::io::Error) -> Self {
        SiteGenerationError::IOError(error)
    }
}

impl From<siru::yaml::Error> for SiteGenerationError {
    fn from(error: siru::yaml::Error) -> Self {
        SiteGenerationError::LanguageError {
            language: "YAML".to_string(),
            error_message: error.to_string(),
            cause: error.source().map(<dyn Error>::to_string),
        }
    }
}

impl From<askama::Error> for SiteGenerationError {
    fn from(error: askama::Error) -> Self {
        SiteGenerationError::LanguageError {
            language: "template".to_string(),
            error_message: error.to_string(),
            cause: error.source().map(<dyn Error>::to_string),
        }
    }
}

impl From<regex::Error> for SiteGenerationError {
    fn from(error: regex::Error) -> Self {
        SiteGenerationError::LanguageError {
            language: "regex".to_string(),
            error_message: error.to_string(),
            cause: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, SiteGenerationError>;
