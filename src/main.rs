mod squeal{
    pub mod create_tables;
    pub mod insert_task;
}
mod errors;
mod models;
mod cli;
mod commands{
    pub mod cmd_do;
}

use clap::Parser;
use cli::JNEL;
use std::{cell::RefCell};

use crate::{commands::cmd_do, errors::TaskError};

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
    pub fn init(){
        // Initialize the db connection, load predefined tags and goals and return the connection.
        let conn = Connection::open("jnel.sqlite").expect("Unable to open DB");    

        create_tables::create_tables().expect("Unable to create DB");
        let mut stmt_tags = conn.prepare("SELECT tag_name FROM tags;").unwrap();
        let mut stmt_goals = conn.prepare("SELECT goal_name FROM goals;").unwrap();
        let mut stmt_datetimes = conn.prepare("SELECT shorthand FROM duration_shorthands;").unwrap();

        let tags_iter = stmt_tags.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap();

        let goals_iter = stmt_goals.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap();

        let datetimes_iter = stmt_datetimes.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap();

        let tags_names: Vec<String> = tags_iter.filter_map(Result::ok).collect();
        let goals_names: Vec<String> = goals_iter.filter_map(Result::ok).collect();
        let datetimes_names: Vec<String> = datetimes_iter.filter_map(Result::ok).collect();

        TAGS.with(|tags| {
            tags.borrow_mut().extend(tags_names);
        });

        GOALS.with(|goals| {
            goals.borrow_mut().extend(goals_names);
        });

        DATETIMES.with(|dts| {
            dts.borrow_mut().extend(datetimes_names);
        });
    }
}

fn main() {

    let args = JNEL::parse();
    init::init();

    let result = match args.clone().mode {
        cli::Mode::Do(cmd) =>{
            match cmd {
                cli::DoCmd::New{..}=> cmd_do::do_new(args),
            }
        }
        _ => Err(TaskError::Mysterious),
    };

                        
    //let task = Task::new(title, priority, due_by, group);
    // create::init_to_db(task, conn);

    println!("holy");
}
