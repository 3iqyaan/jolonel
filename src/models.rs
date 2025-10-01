use chrono;
use clap::{Args, ValueEnum};
use crate::errors::{TaskError, Result};
use std::cell::RefCell;
use crate::{TAGS, GOALS};
pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: Priority,
    pub due_by: chrono::Duration,
    pub recurr: Recurrence,
    pub custom_recurr: chrono::Duration,
    pub state: State,
    pub goal: String,
    pub tags: String,
}

pub struct TaskEvents{
    id: u32,
    task_id: u32,
    state: State,
    timestamp: chrono::NaiveDateTime,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Recurrence{
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

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum State {
    Pending,
    Started,
    Paused,
    Done,
}

impl Task{
    pub fn new(
        title: String, 
        priority: Option<Priority>, 
        due_by: Option<chrono::Duration>, 
        recurr: Option<Recurrence>,
        custom_recurr: Option<chrono::Duration>,
        state: Option<State>,
        goal: Option<String>,
        tags: Option<String>,
    ) -> Result<Task> {
    Ok(Task {
        id: 0,
        title,
        priority: priority.unwrap_or(Priority::Low),
        due_by: due_by.unwrap_or(chrono::Duration::MAX),
        recurr: recurr.unwrap_or(Recurrence::None),
        custom_recurr: custom_recurr.unwrap_or(chrono::Duration::zero()),
        state: state.unwrap_or(State::Pending),
        goal : goal.unwrap_or_default(),
        tags : tags.unwrap_or_default(),
        })
    }
}