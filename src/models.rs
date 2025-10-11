use chrono::{self, NaiveDate, NaiveDateTime, NaiveTime};
use clap::{ValueEnum};
use crate::errors::{Result};

#[derive(Debug)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub priority: Priority,
    pub due_by: chrono::NaiveDateTime,
    pub recur: Recur,
    pub at_time: chrono::NaiveTime,
    pub state: State,
    pub goal: i32, // id of the goal in the db and the hashmap
    pub tag: Vec<i32>, // id of tags in the db and the hashmap
}

pub struct TaskEvents{
    pub id: i32,
    pub task_id: i32,
    pub state: State,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Recur{
    Daily,
    Weekly,
    Monthly,
    Yearly,
    None,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority{
    pub fn to_str(&self) -> String{
            match self{
            Priority::High => String::from("High"),
            Priority::Medium => String::from("Medium"),
            Priority::Low => String::from("Low")
        }
    }
}

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum State {
    Scheduled,
    Pending,
    Doing,
    Paused,
    Completed,
}

impl State{
    pub fn to_str(&self) -> String{
        match self{
            State::Scheduled => String::from("Scheduled"),
            State::Pending => String::from("Pending"),
            State::Doing => String::from("Doing"),
            State::Paused => String::from("Paused"),
            State::Completed => String::from("Completed")
        }
    }
}