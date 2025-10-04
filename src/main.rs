mod model_squeal{
    pub mod insert_task;
}
mod cli_model{
    pub mod do_new;
}
mod errors;
mod models;
mod cli;


use clap::Parser;
use cli::JNEL;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::{cell::RefCell};
use std::sync::Mutex;
use lazy_static::lazy_static;

// use crate::cli_model::do_new::do_new;
use crate::errors::{Result, TaskError};
use crate::cli_model::{do_new};
use crate::models::Task;

lazy_static! {
    pub static ref TAGS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
    pub static ref GOALS: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
    pub static ref DATETIMES: Mutex<HashMap<String, i64>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await?;

    async fn migrate(pool: &PgPool) -> Result<()> {
        sqlx::migrate!().run(pool).await.expect("ERROR");
        Ok(())
    }

    migrate(&pool).await?;

    init::populate_globals(&pool).await?;

    let args = JNEL::parse();

    let _ = match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{..}=> do_new::do_new( pool, args),
            }
        }
        _ => Err(TaskError::Mysterious),
    };

    println!("holy");
    Ok(())
}


pub mod init{

    use std::collections::HashMap;
    use super::{TAGS, GOALS, DATETIMES};
    use sqlx::PgPool;
    use crate::errors::Result;

    pub async fn populate_globals(pool :&PgPool) -> Result<()> {
        refresh_tags(pool);
        refresh_goals(pool);
        refresh_dts(pool);
        Ok(())
    }

    pub async fn refresh_tags(pool :&PgPool) -> Result<()>{

        let rows = sqlx::query!("SELECT id, tag_name FROM tags")
            .fetch_all(pool)
            .await?;

        let mut new_map = HashMap::new();
        for row in rows{
            new_map.insert(row.tag_name, row.id as u32);
        }

        {
            let mut tags = TAGS.lock()?;
            *tags = new_map;
        }

        Ok(())
    }

    pub async fn refresh_goals(pool :&PgPool) -> Result<()>{

        let rows = sqlx::query!("SELECT id, goal_name FROM goals")
            .fetch_all(pool)
            .await?;

        let mut new_map = HashMap::new();
        for row in rows{
            new_map.insert(row.goal_name, row.id as u32);
        }

        {
            let mut dts = GOALS.lock()?;
            *dts = new_map;
        }

        
        Ok(())
    }

    pub async fn refresh_dts(pool :&PgPool) -> Result<()>{

        let rows = sqlx::query!("SELECT shorthand, duration FROM duration_shorthands")
            .fetch_all(pool)
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
