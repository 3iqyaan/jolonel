use std::{io};
use chrono;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum TaskError{

    #[error("Date/time parsing failed : {0}")]
    ChronoParsing(#[from] chrono::ParseError),
    
    #[error("Date/time computing failed (Out of range): {0}")]
    ChronoOutOfRange(#[from] chrono::OutOfRangeError),

    #[error("Parsing month failed: {0}")]
    ChronoMonth(#[from] chrono::ParseMonthError),

    #[error("Parsing weekday failed: {0}")]
    ChronoWeekday(#[from] chrono::ParseWeekdayError),

    #[error("Failed to acquire lock due to poisoning: {0}")]
    Poisoned(String),

    #[error("Sqlx failed: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Failed to read from file")]
    FileParseFailed(#[from] io::Error),

    #[error("Did not recieve handle")]
    JoinError(#[from] JoinError),

    #[error("Goal is not predefined: {0}")]
    InvalidGoal(String),

    #[error("Tag not predefined: {0}")]
    InvalidTag(String),

    #[error("Due shorthand not predefined: {0}")]
    InvalidDue(String),

    #[error("This function should'nt have been called: {0}")]
    WrongWay(String),

    #[error("Disparity between memory and database: {0}")]
    Disparity(String),

    #[error("Dont know what caused this")]
    Mysterious(String)
}

impl<T> From<std::sync::PoisonError<T>> for TaskError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        TaskError::Poisoned(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TaskError>;