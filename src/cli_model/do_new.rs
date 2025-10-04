use crate::models::{Recur, Task, State};
use crate::errors::{Result, TaskError};
use crate::{cli, GOALS, TAGS};
use chrono::{Duration, Local, NaiveDateTime, NaiveTime, Utc};
use sqlx::PgPool;
use crate::model_squeal::{insert_task};
use crate::DATETIMES;
use crate::init::{refresh_dts, refresh_goals, refresh_tags};
use std::ops::Add;
use std::collections::HashMap;
use std::sync::PoisonError;

pub async fn do_new(pool: PgPool, args: cli::JNEL) -> Result<Task> {
    let new_task = match &args.mode {
        cli::Mode::Do(cmd) => match &cmd {
            &cli::DoCmd::New { title, priority, due,recur, state, goal, tag} => {

                // Convert parsed due by data into Option<NaiveDateTime> to be passed into Task::new()
                let due_opt = match due{
                    Some(d) => {
                        match (d.due_short.clone(), d.due_datetime.clone()){
                            (Some(shorthand), None) => { // Case 1: Passed shorthand is predefined
                                let datetimes_lock = DATETIMES.lock().expect("Lock of cached Datetime shorthands poisoned X_X!");
                                if datetimes_lock.contains_key(&shorthand) {
                                    let dur_sec = match datetimes_lock.get(&shorthand){
                                        Some(dur) => *dur,
                                        None => 9223372036854775807,
                                    };
                                    let now = Local::now().naive_local();
                                    let dt = now + Duration::seconds(dur_sec);
                                    Ok(Some(dt))
                                }
                                else {
                                    Err(TaskError::InvalidDue(shorthand))
                                }
                            }
                            (None, Some(dt)) => { // Case 2: due was passed as DateTime
                                let dt = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H-%M-%S")?;
                                Ok(Some(dt))
                            }
                            _ => Ok(None), // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                        }
                    },
                    None => Ok(None) // Case 4: No args were passed into the DueArgs
                };

                // Converting the recur:Option<Recurrence> into recur_type:Option<Recur> and recur_time:Option<NaiveTime>
                // to be passed into Task::new(..recur, at_time..) respectively
                let recur_time: Option<NaiveTime>;
                let recur_type: Option<Recur>;
                match recur{
                    Some(r) => {
                        recur_time = r.at_time;
                        recur_type = r.recur;
                    }
                    None => {
                        recur_time = None;
                        recur_type = None;
                    }
                };

                match state {
                    Some(State::Pending) => {

                    }
                    Some(State::Doing) => {
                        
                    }
                    Some(State::Paused) => {
                        
                    }
                    Some(State::Done) => {
                        
                    }
                    None => {

                    }
                };

                // Convert parsed goal and tag data into Option<String>, insert new string to TAGS and GOALS if needed
                let goal_opt = match goal{
                    Some(g) => {
                        match (g.goal.clone(), g.new_goal.clone()){
                            (Some(name), None) => { // Case 1: Passed goal is predefined
                                let goals_lock = GOALS.lock().expect("Lock of cached goals poisoned X_X!");
                                if goals_lock.contains_key(&name) {
                                    Ok(Some(name))
                                }
                                else {
                                    Err(TaskError::InvalidGoal(name))
                                }
                            }
                            (None, Some(new_goal)) => { // Case 2: Create a new goal

                                sqlx::query!("INSERT INTO goals (goal_name) VALUES ($1)",new_goal);

                                refresh_goals(&pool);

                                Ok(Some(new_goal))
                            }
                            _ => Ok(None), // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                        }
                    },
                    None => Ok(None) // Case 4: No args were passed into the GoalArgs
                };
                let tag_opt = match tag{
                    Some(t) => {
                        match (t.tag.clone(), t.new_tag.clone()){
                            (Some(name), None) => { // Case 1: Passed tag is predefined
                                let tags_lock = TAGS.lock().expect("Lock of cached Tags poisoned X_X!");
                                if tags_lock.contains_key(&name) {
                                    Ok(Some(name))
                                }
                                else {
                                    Err(TaskError::InvalidGoal(name))
                                }
                            }
                            (None, Some(new_tag)) => { // Case 2: Create a new tag
                                
                                sqlx::query!("INSERT INTO tags (tag_name) VALUES ($1)",new_tag);

                                refresh_tags(&pool);

                                Ok(Some(new_tag))
                            }
                            _ => Ok(None), // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
                        }
                    },
                    None => Ok(None) // Case 4: No args were passed into the TagArgs
                };
                let tag_opt = tag_opt?;
                let goal_opt = goal_opt?;
                let due_opt = due_opt?;
                
                Task::new(title.to_string(), priority.clone(), due_opt, recur_type.clone(), recur_time.clone(), None, state.clone(), goal_opt, tag_opt)

            }
        }
        _ => return Err(TaskError::WrongWay(String::from("Mode did not match Do, and we are in - fn do_new()")))
        
    };
    let new_task = insert_task::in_task(pool, new_task).await;
    new_task
    
    
}