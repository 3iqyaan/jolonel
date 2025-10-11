mod squeal;
mod errors;
mod models;
mod cli;


use chrono::{NaiveDateTime, NaiveTime};
use clap::Parser;
use cli::JNEL;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

// use crate::cli_model::do_new::do_new;
use crate::errors::{Result, TaskError};
use crate::models::Task;
use crate::squeal::{helpers, recur_helpers};

lazy_static! {
    pub static ref TAGS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    pub static ref GOALS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    pub static ref DURATION: Mutex<HashMap<String, i64>> = Mutex::new(HashMap::new());
    pub static ref DBURL: String  = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set").to_string();
}

pub fn retrieve_values<'a, K, V>(map: &'a HashMap<K, V>, keys: &[K]) -> Vec<&'a V>
where
    K: Eq + std::hash::Hash,
{
    keys.iter().filter_map(|key| map.get(key)).collect::<Vec<&V>>()
}


const DEFAULT_DUE: NaiveDateTime = NaiveDateTime::new(
    chrono::NaiveDate::from_ymd_opt(3141, 05, 09).unwrap(),
    NaiveTime::from_hms_opt(23, 59, 59)
    .unwrap());

const DEFAULT_TIME: NaiveTime = NaiveTime::from_hms_opt(9, 00, 00).unwrap();



#[tokio::main]
async fn main() -> Result<()> {
    
    let pool = PgPool::connect(&DBURL).await?;

    async fn migrate(pool: &PgPool) -> Result<()> {
        sqlx::migrate!().run( pool).await.expect("ERROR");
        Ok(())
    }

    migrate(&pool).await?;
    populate_globals(&pool).await?;

    let args = JNEL::parse();
    let task = match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{..} => helpers::main::main(cmd).await
            }
        }
        _ => Err(TaskError::Mysterious(String::from("IDK"))),
    }?;

    println!("the task: {:?}", task);
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
        let mut tags = TAGS.lock()?;
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
        let mut dts = GOALS.lock()?;
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
        let mut dts = DURATION.lock()?;
        *dts = new_map;
    }

    
    Ok(())
}

