use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::mpsc;
use crate::{errors::Result};

/// Behaves differently based on the message.
/// Currently, accepts messages from goal_helper, tag_helper, due_helper, and events_helper.
pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Msg>) -> Result<()>{
    while let Some(msg) = rx.recv().await{
        println!("assemble started");
        match msg{
            Msg::Goal { task_id, goal_id } => {
                if goal_id != 0 {
                    sqlx::query(
                        r#"
                        UPDATE tasks
                        SET goal_id = $1
                        WHERE id = $2
                        "#
                    ).bind(goal_id)
                    .bind(task_id)
                    .execute(&pool)
                    .await?;
                }
            }
            Msg::Tag { task_id, tag_ids } => {
                if tag_ids.as_slice() != &[0] {
                    sqlx::query(
                        r#"
                    INSERT INTO task_tags (task_id, tag_id)
                    SELECT $1, unnested_tag_ids
                    FROM unnest($2::int[]) AS unnested_tag_ids
                    ON CONFLICT (task_id, tag_id) DO NOTHING
                                "#
                    ).bind(task_id)
                    .bind(tag_ids)
                    .execute(&pool)
                    .await?;
                }
                
            }
            Msg::Due { task_id, parsed_due } => {
                sqlx::query(
                    r#"
                    UPDATE tasks
                    SET due_by = $1::timestamptz
                    WHERE id = $2;
                    "#
                ).bind(parsed_due.to_string())
                .bind(task_id)
                .execute(&pool)
                .await?;
            }
            Msg::State { task_id, state } => {
                sqlx::query(r#"
                UPDATE tasks
                SET state = $1::task_state
                WHERE id = $2;
                "#
                ).bind(state)
                .bind(task_id)
                .execute(&pool)
                .await?;
            }
            Msg::Priority { task_id, priority } => {
                sqlx::query(
                    r#"
                    UPDATE tasks
                    SET priority = $1::priority_level
                    WHERE id = $2;
                    "#
                ).bind(priority)
                .bind(task_id)
                .execute(&pool)
                .await?;
            }
            Msg::Clockout => {
                rx.close();
            }
        }
    }
    Ok(())
}

pub enum Msg{
    Goal{ task_id: i32, goal_id: i32},
    Tag{ task_id: i32, tag_ids: Vec<i32>},
    Due{ task_id: i32, parsed_due: DateTime<Utc>},
    State{ task_id: i32, state: String},
    Priority{ task_id: i32, priority: String},
    Clockout
}