use std::sync::Arc;

use sqlx::{PgPool, Row};
use tokio::sync::mpsc;

use crate::cli::DoCmd;
use crate::errors::Result;
use crate::squeal::helpers::tag_helper;
use crate::DBURL;


pub async fn main(cmd: DoCmd) -> Result<()>{

    // Create a pool here just to Initialise the task, nothing else
    let pool = PgPool::connect(&DBURL.lock()?).await?;
    
    match cmd {
        DoCmd::New { title, priority, due, recur, state, goal, tag } => {
            // First check if the task is recurring. If it is, give the task to the scheduler's main

            // Create an id for the task
            let row = sqlx::query("INSERT INTO tasks (title) VALUES ($1) RETURNING id;").bind(title).fetch_one(&pool).await?;
            let id: i32 = row.try_get("id")?;

            // Create channels between the main and helper functions.
            
            // First create a channel between `main` and `tag_helper`
            // Send the task id and `Option<TagArgs>`,
            // I think sending the obtained data as is to the spawned reciever and parsing in that helper is ok as I would have to parse the data anyways,
            // and doing it in the same file would help modularise the work from this main.
            let (tx, rx) = mpsc::channel(10);
            tx.send(Arc::new((id, tag))).await.expect("couldnt send msg");
            tag_helper::main(rx).await?;
            
            ()
        }
        _ => ()
    }
    Ok(())
}