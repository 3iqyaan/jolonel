use chrono::{Utc, DateTime};

pub mod due_helper;
pub mod tag_helper;
pub mod goal_helper;
pub mod events_helper;

pub enum Msg{
    Goal{ task_id: i32, goal_id: i32},
    Tag{ task_id: i32, tag_ids: Vec<i32>},
    Due{ task_id: i32, parsed_due: DateTime<Utc>},
    State{ task_id: i32, state: String},
    Priority{ task_id: i32, priority: String}
}