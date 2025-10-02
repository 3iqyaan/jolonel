// use chrono::Local;
// use rusqlite::{params, Connection};
// use crate::models::{Priority, Task};
// use crate::errors::Result;

// pub fn in_task(conn: &Connection, res_task: Result<Task>) -> Result<Task>{
//     let task = res_task?;

//     let priority_str = task.priority.to_str();

//     let due_by_str = task.due_by.format("%Y-%m-%d %H:%M:%S").to_string();

//     let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

//     let recurr_sec = match task.recur{
        
//     };

//     conn.execute("INSERT INTO tags (title, priority, due_by, recurrence, state) 
//         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
//         params![task.title, priority_str, due_by_str, created_at, , "NotStarted", goal_id])?;
    
//     Ok((task))
// }