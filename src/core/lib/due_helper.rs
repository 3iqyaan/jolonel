use std::sync::Arc;
use crate::{cli::DueArgs, LOCAL_DEFAULT, OFFSET};
use super::Msg;
use crate::DURATION;
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use sqlx::PgPool;
use tokio::sync::mpsc;
use crate::errors::{Result, TaskError};

pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Arc<(i32, Option<DueArgs>)>>, tx_asm: mpsc::Sender<Msg>) -> Result<()>{

    while let Some(msg) = rx.recv().await{
        println!("due started");      
   
        let (task_id, due) = &*msg;

        let parsed_due = match due{
            Some(d) => {
                match (&d.due_short, &d.due_datetime){

                    (Some(shorthand), None) => { // Case 1: Passed shorthand is predefined
                        let datetimes_lock = DURATION.lock().await;
                        if datetimes_lock.contains_key(shorthand) {
                            let dur_sec = match datetimes_lock.get(shorthand){
                                Some(dur) => *dur,
                                None => 0,
                            };
                            let now = Utc::now();
                            let dt = now + Duration::seconds(dur_sec);
                            dt
                        }
                        else {
                            *LOCAL_DEFAULT
                        }
                    }
                    (None, Some(dt)) => { // Case 2: due was passed as DateTime
                        let dt = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")?;
                        let offset_dt: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(dt, *OFFSET);
                        let utc_dt = offset_dt.with_timezone(&Utc);
                        utc_dt
                    }
                    _ => *LOCAL_DEFAULT, // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                }
                
            },
            None => *LOCAL_DEFAULT // Case 4: No args were passed into the DueArgs
        };
        tx_asm.send(Msg::Due { task_id: task_id.clone(), parsed_due: parsed_due}).await.unwrap();
    }
    drop(tx_asm);
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