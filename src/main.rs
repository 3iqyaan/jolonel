mod squeal{
    pub mod create_tables;
    pub mod insert_task;
}
mod commands{
    pub mod cmd_do;
}
mod errors;
mod models;
mod cli;


use clap::Parser;
use cli::JNEL;
use rusqlite::Connection;
use std::{cell::RefCell};

use crate::errors::{Result, TaskError};
use crate::commands::{cmd_do};

#[allow(dead_code)]

thread_local! {
    pub static TAGS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    pub static GOALS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    pub static DATETIMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub mod init{
    use super::{TAGS, GOALS, DATETIMES};
    use rusqlite::Connection;
    use crate::squeal::create_tables;
    use crate::errors::{TaskError, Result};
    pub fn init(conn : &Connection) -> Result<()>{

        // Initialize the db connection, load predefined tags and goals and return the connection. 

        create_tables::create_tables(&conn).expect("Unable to create DB");
        let mut stmt_tags = conn.prepare("SELECT tag_name FROM tags;")?;
        let mut stmt_goals = conn.prepare("SELECT goal_name FROM goals;")?;
        let mut stmt_datetimes = conn.prepare("SELECT shorthand FROM duration_shorthands;")?;

        let tags_names: Vec<String> = stmt_tags
            .query_map([], |row| Ok(row.get::<_, String>(0)?))? 
            .map(|r| r.map_err(TaskError::from))              
            .collect::<Result<Vec<_>>>()?;                 

        let goals_names: Vec<String> = stmt_goals
            .query_map([], |row| Ok(row.get::<_, String>(0)?))? 
            .map(|r| r.map_err(TaskError::from))              
            .collect::<Result<Vec<_>>>()?;                 

        let datetimes_names: Vec<String> = stmt_datetimes
            .query_map([], |row| Ok(row.get::<_, String>(0)?))? 
            .map(|r| r.map_err(TaskError::from))              
            .collect::<Result<Vec<_>>>()?;                 


        TAGS.with(|tags| {
            tags.borrow_mut().extend(tags_names);
        });

        GOALS.with(|goals| {
            goals.borrow_mut().extend(goals_names);
        });

        DATETIMES.with(|dts| {
            dts.borrow_mut().extend(datetimes_names);
        });
        Ok(())
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("jnel.sqlite")?;

    let args = JNEL::parse();
    init::init(&conn);

    match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{..}=> cmd_do::do_new(&conn, args),
            }
        }
        _ => Err(TaskError::Mysterious),
    };

                        
    //let task = Task::new(title, priority, due_by, group);
    // create::init_to_db(task, conn);

    println!("holy");
    Ok(())
}
