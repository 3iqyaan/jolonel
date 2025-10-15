use tokio::sync::{mpsc, broadcast};
use sqlx::PgPool;

use crate::core::lib::Msg;
use crate::errors::{Result, TaskError};

pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Msg>, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()>{
    let mut results = vec![];
    loop{
        let pool_ = pool.clone();
        tokio::select! {
            Some(msg) = rx.recv() => {
                results.push(msg.process_sch(pool_).await);
            }
            _ = shutdown_rx.recv() => {
                while let Some(msg) = rx.recv().await {
                    let pool_1 = pool_.clone();
                    results.push(msg.process_sch(pool_1).await);
                }
                rx.close();
                break;
            }
        }
    }
    let mut ok = true;
    for result in results{
        if let Err(_e) = result {
            ok = false;
        }
    }
    if ok == true {Ok(())}
    else{Err(TaskError::Mysterious("scheduler couldnt process massages without panicking :(".into()))}
}

impl Msg{
    pub async fn process_sch(&self, pool: PgPool) -> Result<()>{
        match &self{
            Msg::Goal { task_id, goal_id } => {
                if *goal_id != 0 {
                    sqlx::query(
                        r#"
                        UPDATE scheduled_tasks
                        SET goal_id = $1
                        WHERE id = $2
                        "#
                    ).bind(goal_id)
                    .bind(task_id)
                    .execute(&pool)
                    .await?;
                    Ok(())
                }
                else {
                    Ok(())
                }
            }
            Msg::Tag { task_id, tag_ids } => {
                if tag_ids.as_slice() != &[0] {
                    sqlx::query(
                        r#"
                    INSERT INTO scheduled_task_tags (task_id, tag_id)
                    SELECT $1, unit_tag_ids
                    FROM unnest($2::int[]) AS unit_tag_ids
                    ON CONFLICT (scheduled_task_id, tag_id) DO NOTHING
                                "#
                    ).bind(task_id)
                    .bind(tag_ids)
                    .execute(&pool)
                    .await?;
                    Ok(())
                }
                else{
                    Ok(())
                }
                
            }
            Msg::Due { task_id, parsed_due } => {
                sqlx::query(
                    r#"
                    UPDATE scheduled_tasks
                    SET due_by = $1::timestamptz
                    WHERE id = $2;
                    "#
                ).bind(parsed_due.to_string())
                .bind(task_id)
                .execute(&pool)
                .await?;
                Ok(())
            }
            Msg::Priority { task_id, priority } => {
                sqlx::query(
                    r#"
                    UPDATE scheduled_tasks
                    SET priority = $1::priority_level
                    WHERE id = $2;
                    "#
                ).bind(priority)
                .bind(task_id)
                .execute(&pool)
                .await?;
                Ok(())
            }
            _ => Ok(())
        }
    }
}