use std::sync::Arc;
use sqlx::PgPool;
use tokio::{sync::mpsc, task::JoinSet};
use crate::{errors::Result, models::State, squeal::helpers::assemble::Msg, DBURL};

pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Arc<(i32, Option<State>)>>, tx_asm: mpsc::Sender<Msg>) -> Result<()> {

    let mut join_set = JoinSet::new();

    while let Some(msg) = rx.recv().await {
        println!("event started");
        let pool_clone = pool.clone();
        let tx_clone = tx_asm.clone();

        join_set.spawn(async move{
            let (task_id, state) = &*msg;

            let state = state.unwrap_or_else(|| State::Pending).to_str();

            update_state(&pool_clone, task_id, state.clone()).await.unwrap();

            tx_clone.send(Msg::State { task_id: task_id.clone(), state: state.clone() }).await.unwrap();
        });
    }
    while let Some(_) = join_set.join_next().await{
        ()
    }
    join_set.abort_all();
    Ok(())
}

pub async fn update_state(pool: &PgPool, task_id: &i32, state: String) -> Result<()> {
    sqlx::query(r#"
    INSERT INTO task_events (task_id, state)
    VALUES ($1, $2::task_state);
    "#
    ).bind(task_id)
    .bind(state)
    .execute(pool)
    .await?;

    Ok(())
}