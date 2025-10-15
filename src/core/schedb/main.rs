use std::sync::Arc;
use sqlx::{Row, PgPool};
use tokio::sync::{broadcast, mpsc};

use crate::{cli::DoCmd, errors::Result, core::{lib, schedb}, DBURL};

pub async fn main(cmd: DoCmd) -> Result<()>{
    // The pool to clone and pass to the helpers
    let pool = PgPool::connect(&DBURL).await?;

    match cmd{
        DoCmd::New { title, priority, due, recur, state, goal, tag } => {

            // Create an id for the scheduled task
            let row = sqlx::query("INSERT INTO scheduled_tasks (title) VALUES ($1) RETURNING id;").bind(title).fetch_one(&pool).await?;
            let id: i32 = row.try_get("id")?;

            let (shutdown_tx, _) = broadcast::channel::<()>(1);
            let shutdown_rx = shutdown_tx.subscribe();

            // Create the channel between the helpers and the scheduler
            let (tx_sch, rx_sch) = mpsc::channel::<lib::Msg>(15);
            let pool_ = pool.clone();
            let sch_handle = tokio::spawn(async move{
                if let Err(e) = schedb::scheduler::main(pool_, rx_sch, shutdown_rx).await{
                    eprint!("Scheduler failed: {:?}", e)
                }
            });

            // Create the channels between the the main and the helpers
            // First a channel for the `tag_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_sch.clone();
            let pool_ = pool.clone();
            let tag_handle = tokio::spawn(async move{
                if let Err(e) = lib::tag_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("scheduler's tag helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, tag))).await.expect("Couldn't contact the tag helper");
            
            // Now a channel for the `goal_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_sch.clone();
            let pool_ = pool.clone();
            let goal_handle = tokio::spawn(async move{
                if let Err(e) = lib::goal_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("scheduler's goal helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, goal))).await.expect("Couldn't contact the goal helper");
            
            // Now a channel for the `due_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_sch.clone();
            let pool_ = pool.clone();
            let due_handle = tokio::spawn(async move{
                if let Err(e) = lib::due_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("scheduler's due helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, due))).await.expect("Couldn't contact the deadline helper");

            // Send the priority of the task
            tx_sch.send(lib::Msg::Priority { task_id: id.clone(), priority: priority.unwrap_or_else(|| crate::models::Priority::Low).to_str() })
            .await.expect("Couldn't contact the scheduler");

            goal_handle.await?;
            tag_handle.await?;
            due_handle.await?;


        }
    }

    Ok(())
}