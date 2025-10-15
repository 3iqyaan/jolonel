use std::sync::Arc;
use sqlx::{PgPool, Row};
use tokio::sync::{broadcast, mpsc};

use crate::cli::{DoCmd};
use crate::errors::{Result};
use crate::core::db::assemble;
use crate::core::lib;
use crate::{DBURL};


pub async fn main(cmd: DoCmd) -> Result<()>{

    // Create a pool here to clone and pass to the helpers
    let pool = PgPool::connect(&DBURL).await?;
    
    match cmd {
        DoCmd::New { title, priority, due, recur: _, state, goal, tag } => {
            
            // Create an id for the task
            let row = sqlx::query("INSERT INTO tasks (title) VALUES ($1) RETURNING id;").bind(title).fetch_one(&pool).await?;
            let id: i32 = row.try_get("id")?;

            // Create channels between the main and helper functions.
            
            // First create a channel between other helpers and the thread that updates the `tasks` table

            let (shutdown_tx, _) = broadcast::channel::<()>(1);
            let shutdown_rx = shutdown_tx.subscribe();
            
            let (tx_asm, rx_asm) = mpsc::channel(15);
            let pool_ = pool.clone();
            let asm_handle = tokio::spawn(async move {
                if let Err(e) = assemble::main(pool_, rx_asm, shutdown_rx).await{
                    eprintln!("assembler failed: {:?}", e);
                }
            });
            
            // Now a channel for the `tag_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let tag_handle = tokio::spawn(async move{
                if let Err(e) = lib::tag_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("tag helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, tag))).await.expect("Couldn't contact the tag helper");
            drop(tx);

            // Now a channel for the `goal_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let goal_handle = tokio::spawn(async move{
                if let Err(e) = lib::goal_helper::main(pool_, rx, tx_clone).await{
                    eprint!("goal helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, goal))).await.expect("Couldn't contact the goal helper");
            drop(tx);

            // Now a channel for the `due_helper`
            let (tx, rx) = mpsc::channel(10);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let due_handle = tokio::spawn(async move{
                if let Err(e) = lib::due_helper::main(pool_, rx, tx_clone).await{
                    eprint!("Due helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, due))).await.expect("Couldn't contact the deadline helper");
            drop(tx);

            // Now a channel for the `events_helper`
            println!("about to start events");
            let (tx, rx) = mpsc::channel(15);
            let tx_clone = tx_asm.clone();
            let pool_ = pool.clone();
            let event_handle = tokio::spawn(async move{
                if let Err(e) = lib::events_helper::main(pool_, rx, tx_clone).await{
                    eprintln!("event helper failed: {:?}", e);
                }
            });
            tx.send(Arc::new((id, state))).await.expect("Couldn't contact the events helper");
            drop(tx);

            // Send the priority of the task
            tx_asm.send(lib::Msg::Priority { task_id: id.clone(), priority: priority.unwrap_or_else(|| crate::models::Priority::Low).to_str() })
            .await.expect("Couldn't contact the assembler");

            drop(tx_asm);

            tag_handle.await?;
            goal_handle.await?;
            due_handle.await?;
            event_handle.await?;

            shutdown_tx.send(())?;
            drop(shutdown_tx);

            asm_handle.await?;
        } // the DoCmd::New variant
    } // match cmd
    return Ok(())
}



