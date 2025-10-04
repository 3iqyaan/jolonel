use std::future::Pending;

use chrono::{self, NaiveDate, NaiveDateTime, NaiveTime};
use clap::{ValueEnum};
use crate::errors::{Result};

pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: Priority,
    pub due_by: chrono::NaiveDateTime,
    pub recur: Recur,
    pub at_time: chrono::NaiveTime,
    pub custom_recur: chrono::Duration,
    pub state: State,
    pub goal: String,
    pub tags: String,
}

pub struct TaskEvents{
    pub id: u32,
    pub task_id: u32,
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
    Pending,
    Doing,
    Paused,
    Completed,
}

impl State{
    pub fn to_str(&self) -> String{
        match self{
            State::Pending => String::from("Pending"),
            State::Doing => String::from("Doing"),
            State::Paused => String::from("Paused"),
            State::Completed => String::from("Completed")
        }
    }
}

impl Task{
    pub fn new(
        title: String, 
        priority: Option<Priority>, 
        due_by: Option<chrono::NaiveDateTime>, 
        recur: Option<Recur>,
        at_time: Option<NaiveTime>,
        custom_recur: Option<chrono::Duration>,
        state: Option<State>,
        goal: Option<String>,
        tags: Option<String>,
    ) -> Result<Task> {
    Ok(Task {
        id: 0,
        title,
        priority: priority.unwrap_or(Priority::Low),
        due_by: due_by
            .unwrap_or(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(314159, 12, 31).unwrap(),
                NaiveTime::from_hms_opt(23, 59, 59).unwrap())),
        recur: recur.unwrap_or(Recur::None),
        at_time: at_time.unwrap_or(NaiveTime::from_hms_opt(9, 00, 00).unwrap()),
        custom_recur: custom_recur.unwrap_or(chrono::Duration::zero()),
        state: state.unwrap_or(State::Pending),
        goal : goal.unwrap_or_default(),
        tags : tags.unwrap_or_default(),
        })
    }
}