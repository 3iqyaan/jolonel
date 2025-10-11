// use clap::builder::Str;
// use sqlx::{Postgres, QueryBuilder, Transaction, Row};
// use crate::{refresh_goals, refresh_tags};
// use crate::models::{Recur, Task};
// use crate::errors::Result;

// pub async fn tags_in(txn: &mut Transaction<'_, Postgres>, new_tags: Vec<String>) -> Result<Vec<i32>>{
//     let new_tag_ids = sqlx::query_scalar(
//         r#"
//         WITH input_tags (tag_name) AS (
//             SELECT * FROM unnest($1::text[])
//         )
//         INSERT INTO tags (tag_name)
//         SELECT tag_name FROM input_tags
//         ON CONFLICT (tag_name) DO NOTHING
//         RETURNING id;
//         "#
//     )
//     .bind(new_tags)
//     .fetch_all(&mut **txn)
//     .await?;

//     refresh_tags(pool).await?;
//     Ok(new_tag_ids)
// }

// pub async fn goal_in(txn: &mut Transaction<'_, Postgres>, new_goal: String) -> Result<i32>{
//     let new_goal_id = sqlx::query_scalar(
//         r#"
//         INSERT INTO goals (goal_name)
//         VALUES ($1)
//         ON CONFLICT (goal_name) DO NOTHING
//         RETURNING id
//         "#
//     )
//     .bind(&new_goal)
//     .fetch_one(&mut **txn)
//     .await?;
//     refresh_goals(&mut *txn).await?;
//     Ok(new_goal_id)
// }

// pub async fn init_task(txn: &mut Transaction<'_, Postgres>, title: &String) -> Result<i32> {
//     let row = sqlx::query("INSERT INTO tasks (title) VALUES ($1) RETURNING id;").bind(title).fetch_one(&mut **txn).await?;
//     let id: i32 = row.try_get("id")?;
//     Ok(id)
// }

// pub async fn in_task(txn: &mut Transaction<'_, Postgres>, res_task: Result<Task>) -> Result<Task>{

//     let task = res_task?;

//     let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE tasks SET priority = ");
//     // qb.push_bind(&task.title); qb.push(", ");

//     let priority_str = task.priority.to_str();
//     qb.push_bind(priority_str); qb.push("::priority_level, due_by = ");

//     let due_by_str = task.due_by.format("%Y-%m-%d %H:%M:%S").to_string();
//     qb.push_bind(due_by_str); qb.push("::timestamp, does_recur = ");

//     let does_recur = match task.recur{
//         Recur::None => false,
//         _ => true
//     };

//     qb.push_bind(does_recur); qb.push(", state = ");


//     let state_str = task.state.to_str();
//     qb.push_bind(state_str); qb.push("::task_state, goal_id = ");

//     qb.push_bind(task.goal); qb.push(" WHERE id = ");

//     qb.push_bind(task.id); qb.push(";");
    
//     qb.build().execute(&mut **txn).await?; 

//     Ok(task)
// }

// pub async fn in_task_tag(txn: &mut Transaction<'_, Postgres>, tag_ids: &Vec<i32>, task_id: &i32) -> Result<()>{
//     sqlx::query(
//                 r#"
//         INSERT INTO task_tags (task_id, tag_id)
//         SELECT $1, unnested_tag_ids
//         FROM unnest($2::int[]) AS unnested_tag_ids
//         ON CONFLICT (task_id, tag_id) DO NOTHING;
//                     "#
//         ).bind(task_id)
//         .bind(tag_ids)
//         .execute(&mut **txn)
//         .await?;
//     refresh_tags(None);
//     Ok(())

// }