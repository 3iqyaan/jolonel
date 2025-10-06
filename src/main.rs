mod model_squeal{
    pub mod new_task;
}
mod cli_model{
    pub mod do_new;
}
mod errors;
mod models;
mod cli;


use chrono::{NaiveDateTime, NaiveTime};
use clap::Parser;
use cli::JNEL;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

// use crate::cli_model::do_new::do_new;
use crate::errors::{Result, TaskError};
use crate::cli_model::{do_new};
use crate::models::Task;

lazy_static! {
    pub static ref TAGS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    pub static ref GOALS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    pub static ref DATETIMES: Mutex<HashMap<String, i64>> = Mutex::new(HashMap::new());
}

const DEFAULT_DUE: NaiveDateTime = NaiveDateTime::new(
    chrono::NaiveDate::from_ymd_opt(31415, 12, 31).unwrap(),
    NaiveTime::from_hms_opt(23, 59, 59)
    .unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await?;

    async fn migrate(txn: &mut Transaction<'_, Postgres>) -> Result<()> {
        sqlx::migrate!().run( txn).await.expect("ERROR");
        Ok(())
    }

    let mut txn = pool.begin().await?;
    migrate(&mut txn).await?;
    init::populate_globals(&mut txn).await?;
    txn.commit().await?;

    let mut txn = pool.begin().await?;
    let args = JNEL::parse();
    let task = match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{..} => Task::do_new(&mut txn, cmd).await,
            }
        }
        _ => Err(TaskError::Mysterious),
    }?;
    txn.commit().await?;


    println!("the task: {:?}", task);
    Ok(())
}


pub mod init{

    use std::collections::HashMap;
    use super::{TAGS, GOALS, DATETIMES};
    use sqlx::{Postgres, Transaction};
    use crate::errors::Result;

    pub async fn populate_globals(txn :&mut Transaction<'_, Postgres>) -> Result<()> {
        refresh_tags(&mut *txn).await?;
        refresh_goals(&mut *txn).await?;
        refresh_dts(&mut *txn).await?;
        Ok(())
    }

    pub async fn refresh_tags(txn :&mut Transaction<'_, Postgres>) -> Result<()>{

        let rows = sqlx::query!("SELECT id, tag_name FROM tags")
            .fetch_all(&mut **txn)
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

    pub async fn refresh_goals(txn :&mut Transaction<'_, Postgres>) -> Result<()>{

        let rows = sqlx::query!("SELECT id, goal_name FROM goals")
            .fetch_all(&mut **txn)
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

    pub async fn refresh_dts(txn :&mut Transaction<'_, Postgres>) -> Result<()>{

        let rows = sqlx::query!("SELECT shorthand, duration FROM duration_shorthands")
            .fetch_all(&mut **txn)
            .await?;

        let mut new_map = HashMap::new();
        for row in rows{
            new_map.insert(row.shorthand, row.duration as i64);
        }

        {
            let mut dts = DATETIMES.lock()?;
            *dts = new_map;
        }

        
        Ok(())
    }
}
