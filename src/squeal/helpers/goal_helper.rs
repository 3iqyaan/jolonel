use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use crate::errors::{Result, TaskError};
use crate::squeal::helpers::assemble::{Msg};
use crate::{GOALS, refresh_goals};
use crate::{cli::GoalArgs, DBURL};

/// Parses the `GoalArgs`, creates new ones if need, syncs the cache with the db, assigns the task the parsed goal
pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Arc<(i32, Option<GoalArgs>)>>, tx_asm: mpsc::Sender<Msg>) -> Result<()> {

    let mut join_set: JoinSet<std::result::Result<(), TaskError>> = JoinSet::new();

    while let Some(msg) = rx.recv().await{
        println!("goal started");
        let pool_clone = pool.clone();
        let tx_clone = tx_asm.clone();

        join_set.spawn(async move {
            let (task_id, goal) = &*msg;
            
            // Parse the `GoalArgs`, store in db and refresh cache if it is a new one, obtain its id
            // ---
            let goal_id = match goal{
                Some(g) => {
                    match (&g.goal, &g.new_goal){

                        (Some(name), None) => { // Case 1: Passed goal is predefined
                            let goals_lock = GOALS.lock()?;

                            if let Some(goal_id) = goals_lock.get(name) { Ok(*goal_id) }
                            else { println!("Passed goal was not predefined, to create a new goal, use the `-G` flag"); Ok(0) }
                        }
                        (None, Some(new_goal)) => { // Case 2: Create a new goal
                            save_goal(&pool_clone.clone(), &new_goal).await
                        }
                        _ => Ok(0), // Case 3: Both was passed or neither, 
                                    // both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                    }
                },
                None => Ok(0) // Case 4: No args were passed into the GoalArgs
            }?;
            // ---

            // Now, connect the goal and the task by updating the `goal_id` field in the `tasks` table
            // ---
            tx_clone.send(Msg::Goal { task_id: task_id.clone(), goal_id: goal_id.clone() }).await.unwrap();
            // ---
            Ok(())
        });
    } 
    while let Some(_) = join_set.join_next().await{
        ()
    }
    join_set.abort_all();
    Ok(())
}

/// Store the passed goal in the db,
/// Sync the cache with the 
/// Return the `new_goal_id`
/// 
/// *note: When a predefined goal is passed as new, it is ignored and no action is taken*
pub async fn save_goal(pool: &PgPool, new_goal: &String) -> Result<i32>{
    let new_goal_id = sqlx::query_scalar(
        r#"
        INSERT INTO goals (goal_name)
        VALUES ($1)
        ON CONFLICT (goal_name) DO NOTHING
        RETURNING id
        "#
    )
    .bind(&new_goal)
    .fetch_one(pool)
    .await?;

    refresh_goals(&pool).await?;

    Ok(new_goal_id)
}