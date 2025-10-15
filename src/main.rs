mod core;
mod errors;
mod models;
mod cli;

use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, NaiveTime, TimeZone, Utc};
use clap::Parser;
use cli::JNEL;
use sqlx::{PgPool};
use tokio::sync::mpsc;
use tokio::time::sleep_until;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::{Mutex};
use std::sync::Arc;
use lazy_static::lazy_static;

use crate::cli::Recurrence;
use crate::errors::{Result, TaskError};
use crate::core::{db, schedb};

lazy_static! {
    pub static ref TAGS: Arc<Mutex<HashMap<String, i32>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref GOALS: Arc<Mutex<HashMap<String, i32>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref DURATION: Arc<Mutex<HashMap<String, i64>>> = Arc::new(Mutex::new(HashMap::new())); 
    pub static ref DBURL: String  = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set").to_string();
    pub static ref OFFSET: FixedOffset = *Local::now().offset();
    pub static ref DEFAULT_DAY: DateTime<FixedOffset> = OFFSET.with_ymd_and_hms(3141, 05, 09, 02, 06, 53).unwrap();
    pub static ref LOCAL_DEFAULT: DateTime<Utc> = DEFAULT_DAY.with_timezone(&Utc);
}
/// Retrieves values from a hashmap using the `filter_map().collect()` method
pub fn retrieve_values<'a, K, V>(map: &'a HashMap<K, V>, keys: &[K]) -> Vec<&'a V> where K: Eq + std::hash::Hash{
    keys.iter().filter_map(|key| map.get(key)).collect::<Vec<&V>>()
}

pub enum Notif{
    NewEarlier,
    Alarm
}

/// - Triggered upon reaching the scheduled/recurring task DateTime, OR when an earlier schedule/recurrence is added
/// - Retrieves the id of the task in the `scheduled_tasks`, informs the assembler, calculates the next earliest schedule/recurrence, sleeps until then
/// - Only changes its sleep duration if it was woken by an earlier schedule/recurrence.
// pub async fn schedule_dispatch(mut rx: mpsc::Receiver<Notif>){
//     let next_time = schedules
//     tokio::select! {
//         Some(time) = rx.recv() => {
            
            
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let pool = PgPool::connect(&DBURL).await?;

    async fn migrate(pool: &PgPool) -> Result<()> {
        sqlx::migrate!().run( pool).await.expect("ERROR");
        Ok(())
    }

    migrate(&pool).await?;
    populate_globals(&pool).await?;

    // let (tx_dispatch, rx_dispatch) = mpsc::channel(10);
    // tokio::spawn(async move{
    //     schedule_dispatch(rx_dispatch);
    // });


    let args = JNEL::parse();
    let task = match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{recur, ..} => {
                    if is_recur(recur) {
                        schedb::main::main(cmd).await
                    }
                    else {
                        db::main::main(cmd).await
                    }
                }
            }
        }
        _ => Err(TaskError::Mysterious(String::from("IDK"))),
    }?;

    println!("the task: {:?}", task);
    let time = start.elapsed();
    println!("execution took time: {:.3?}", time);
    Ok(())
}

pub async fn populate_globals(pool: &PgPool) -> Result<()> {
    refresh_tags(pool).await?;
    refresh_goals(pool).await?;
    refresh_dts(pool).await?;
    Ok(())
}

pub async fn refresh_tags(pool: &PgPool) -> Result<()>{
    let mut txn = pool.begin().await?;
    
    let rows = sqlx::query!("SELECT id, tag_name FROM tags")
        .fetch_all(&mut *txn)
        .await?;

    let mut new_map = HashMap::new();
    for row in rows{
        new_map.insert(row.tag_name, row.id as i32);
    }

    {
        let mut tags = TAGS.lock().await;
        *tags = new_map;
    }
    Ok(())
}

pub async fn refresh_goals(pool: &PgPool) -> Result<()>{

    let rows = sqlx::query!("SELECT id, goal_name FROM goals")
        .fetch_all(pool)
        .await?;

    let mut new_map = HashMap::new();
    for row in rows{
        new_map.insert(row.goal_name, row.id as i32);
    }

    {
        let mut dts = GOALS.lock().await;
        *dts = new_map;
    }

    
    Ok(())
}

pub async fn refresh_dts(pool: &PgPool) -> Result<()>{

    let rows = sqlx::query!("SELECT shorthand, duration FROM duration_shorthands")
        .fetch_all(pool)
        .await?;

    let mut new_map = HashMap::new();
    for row in rows{
        new_map.insert(row.shorthand, row.duration as i64);
    }

    {
        let mut dts = DURATION.lock().await;
        *dts = new_map;
    }

    
    Ok(())
}

pub fn is_recur(recur: Option<Recurrence>) -> bool{

    match recur{
        Some(ref r) =>{
            match (r.recur, r.at_time){
                (Some(_), Some(_)) => true,
                (Some(_), None) => true,
                (None, Some(_)) => true,
                (None, None) => false
            }
        }
        None => false
    }
}

