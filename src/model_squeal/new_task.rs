use sqlx::{Postgres, QueryBuilder, Transaction};
use crate::init::{refresh_goals, refresh_tags};
use crate::models::{Recur, Task};
use crate::errors::Result;

pub async fn tag_in(tx: &mut Transaction<'_, Postgres>, new_tag: String) -> Result<Option<i32>>{
    let new_tag_id = sqlx::query_scalar(
        r#"
        INSERT INTO tags (tag_name)
        VALUES ($1)
        ON CONFLICT (tag_name) DO UPDATE
        SET tag_name = EXCLUDED.tag_name
        RETURNING id
        "#
    )
    .bind(&new_tag)
    .fetch_one(&mut **tx)
    .await?;

    refresh_tags(&mut *tx).await?;
    Ok(Some(new_tag_id))
}

pub async fn goal_in(tx: &mut Transaction<'_, Postgres>, new_goal: String) -> Result<Option<i32>>{
    let new_goal_id = sqlx::query_scalar(
        r#"
        INSERT INTO goals (goal_name)
        VALUES ($1)
        ON CONFLICT (goal_name) DO UPDATE
        SET goal_name = EXCLUDED.goal_name
        RETURNING id
        "#
    )
    .bind(&new_goal)
    .fetch_one(&mut **tx)
    .await?;
    refresh_goals(&mut *tx).await?;
    Ok(Some(new_goal_id))
}

pub async fn in_task(tx: &mut Transaction<'_, Postgres>, res_task: Result<Task>) -> Result<Task>{

    let task = res_task?;

    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("INSERT INTO tasks (title, priority, due_by, does_recur, state, goal_id) VALUES (");
    qb.push_bind(&task.title); qb.push(", ");

    let priority_str = task.priority.to_str();
    qb.push_bind(priority_str); qb.push("::priority_level, ");

    let due_by_str = task.due_by.format("%Y-%m-%d %H:%M:%S").to_string();
    qb.push_bind(due_by_str); qb.push("::timestamp, ");

    let does_recur = match task.recur{
        Recur::None => false,
        _ => true
    };

    let does_recur = if !does_recur {
        if chrono::TimeDelta::is_zero(&task.custom_recur){
            false
        }
        else {true}
    
    }   
    else{
        true
    };
    qb.push_bind(does_recur); qb.push(", ");


    let state_str = task.state.to_str();
    qb.push_bind(state_str); qb.push("::task_state, ");

    qb.push_bind(task.goal); qb.push(")");
    
    qb.build().execute(&mut **tx).await?; 

    Ok(task)
}