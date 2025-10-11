use std::sync::Arc;
use crate::cli::DueArgs;
use crate::squeal::helpers::assemble::{self, Msg};
use crate::DBURL;
use crate::DURATION;
use chrono::{NaiveDateTime, Duration, Local};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use crate::DEFAULT_DUE;
use crate::errors::{Result, TaskError};

pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Arc<(i32, Option<DueArgs>)>>, tx_asm: mpsc::Sender<Msg>) -> Result<()>{

    let mut join_set: JoinSet<std::result::Result<NaiveDateTime, TaskError>> = JoinSet::new();

    while let Some(msg) = rx.recv().await{
        println!("due started");
        let tx_clone = tx_asm.clone();

        join_set.spawn(async move{
            let (task_id, due) = &*msg;

            let parsed_due = match due{
                Some(d) => {
                    match (&d.due_short, &d.due_datetime){

                        (Some(shorthand), None) => { // Case 1: Passed shorthand is predefined
                            let datetimes_lock = DURATION.lock()?;
                            if datetimes_lock.contains_key(shorthand) {
                                let dur_sec = match datetimes_lock.get(shorthand){
                                    Some(dur) => *dur,
                                    None => 0,
                                };
                                let now = Local::now().naive_local();
                                let dt = now + Duration::seconds(dur_sec);
                                dt
                            }
                            else {
                                DEFAULT_DUE
                            }
                        }
                        (None, Some(dt)) => { // Case 2: due was passed as DateTime
                            let dt = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")?;
                            dt
                        }
                        _ => DEFAULT_DUE, // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                    }
                    
                },
                None => DEFAULT_DUE // Case 4: No args were passed into the DueArgs
            };
            let parsed_due_utc = parsed_due.and_utc();
            tx_clone.send(Msg::Due { task_id: task_id.clone(), parsed_due: parsed_due_utc}).await.unwrap();

            Ok(parsed_due)
        });
        
    }

    while let Some(_) = join_set.join_next().await{
        ()
    }
    join_set.abort_all();
    Ok(())
}

pub async fn save_shorthand(pool: &PgPool, name: &String, dur: &i64) -> Result<()>{
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO duration_shorthands (shorthand, duration)
        VALUES ($1, $2)
        ON CONFLICT (shorthand) DO NOTHING
        RETURNING id;
        "#
        ).bind(name)
        .bind(dur)
        .fetch_one(pool)
        .await?;
    Ok(())
}