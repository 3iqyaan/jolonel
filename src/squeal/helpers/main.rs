use std::sync::Arc;
use sqlx::{PgPool, Row};
use tokio::sync::mpsc;

use crate::cli::{DoCmd};
use crate::errors::{Result};
use crate::squeal::helpers::assemble::Msg;
use crate::squeal::helpers::{assemble, due_helper, events_helper, goal_helper, tag_helper};
use crate::{DBURL};


pub async fn main(cmd: DoCmd) -> Result<()>{

    // Create a pool here to clone and pass to the helpers
    let pool = PgPool::connect(&DBURL).await?;
    
    match cmd {
        DoCmd::New { title, priority, due, recur, state, goal, tag } => {
            // First check if the task is recurring. If it is, give the task to the scheduler's main

            // Create an id for the task
            let row = sqlx::query("INSERT INTO tasks (title) VALUES ($1) RETURNING id;").bind(title).fetch_one(&pool).await?;
            let id: i32 = row.try_get("id")?;

            // Create channels between the main and helper functions.
            
            // First create a channel between other helpers and the thread that updates the `tasks` table
            
                let (tx_asm, rx_asm) = mpsc::channel(15);
                let pool_ = pool.clone();
                let asm_handle = tokio::spawn(async move {
                    if let Err(e) = assemble::main(pool_, rx_asm).await{
                        eprintln!("assembler failed: {:?}", e);
                    }
                });
            

            // Now a channel for the `tag_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let tag_handle = tokio::spawn(async move{
                if let Err(e) = tag_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("tag helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, tag))).await.expect("Couldn't contact the tag helper");
            
            // Now a channel for the `goal_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let goal_handle = tokio::spawn(async move{
                if let Err(e) = goal_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("goal helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, goal))).await.expect("Couldn't contact the goal helper");
            
            // Now a channel for the `due_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let due_handle = tokio::spawn(async move{
                if let Err(e) = due_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("due helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, due))).await.expect("Couldn't contact the deadline helper");

            // Now a channel for the `events_helper`
            let (tx, rx) = mpsc::channel(15);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let event_handle = tokio::spawn(async move{
                if let Err(e) = events_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("event helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, state))).await.expect("Couldn't contact the events helper");
            
            tx_asm.send(assemble::Msg::Priority { task_id: id.clone(), priority: priority.unwrap_or_else(|| crate::models::Priority::Low).to_str() })
            .await.expect("Couldn't contact the assembler");

            tag_handle.await?;
            goal_handle.await?;
            due_handle.await?;
            event_handle.await?;

            tx_asm.send(Msg::Clockout).await.expect("couldn't clockout the assembler");
            asm_handle.await?;
            
        } // the DoCmd::New variant
    } // match cmd
    Ok(())
}

