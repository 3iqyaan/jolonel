use chrono;
use thiserror::Error;
use rusqlite;
#[derive(Debug, Error)]
pub enum TaskError{
    #[error("Database failed: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Date/time parsing failed : {0}")]
    ChronoParsing(#[from] chrono::ParseError),
    
    #[error("Date/time computing failed (Out of range): {0}")]
    ChronoOutOfRange(#[from] chrono::OutOfRangeError),

    #[error("Parsing month failed: {0}")]
    ChronoMonth(#[from] chrono::ParseMonthError),

    #[error("Parsing weekday failed: {0}")]
    ChronoWeekday(#[from] chrono::ParseWeekdayError),

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
    Mysterious
}

pub type Result<T> = std::result::Result<T, TaskError>;