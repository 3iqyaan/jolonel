use chrono::Local;
use sqlx::{PgPool};
use crate::models::{Priority, Task};
use crate::errors::Result;

pub async fn in_task(pool: PgPool, res_task: Result<Task>) -> Result<Task>{
    let task = res_task?;

    let priority_str = task.priority.to_str();

    let due_by_str = task.due_by.format("%Y-%m-%d %H:%M:%S").to_string();

    let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let recurr_sec = match task.recur{
        
    };

    let state_str = task.state.to_str();

    conn.execute("INSERT INTO tags (title, priority, due_by, recurrence, state) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
        params![task.title, priority_str, due_by_str, created_at, recurrence, state, goal_id])?;
    
    Ok((task))
}