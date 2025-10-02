use crate::models::{Recur, Task, State};
use crate::errors::{Result, TaskError};
use crate::{cli, GOALS, TAGS};
use chrono::{Duration, Local, NaiveTime, Utc};
use rusqlite::{Connection, params};
use crate::squeal::{insert_task};
use crate::DATETIMES;
use std::ops::Add;

pub fn do_new(conn: &Connection, args: cli::JNEL) -> Result<Task> {
    let new_task = match &args.mode {
        cli::Mode::Do(cmd) => match &cmd {
            &cli::DoCmd::New { title, priority, due,recur, state, goal, tag} => {

                // Convert parsed due by data into Option<NaiveDateTime> to be passed into Task::new()
                let due = match due {
                    Some(d) => {
                        if let Some(ref db) = d.due_short {
                            if !DATETIMES.with(|dt| dt.borrow().contains(&db)) {
                                return Err(TaskError::InvalidDue(db.clone()));
                            }
                            else{
                                let mut stmt = conn.prepare("SELECT shorthand, duration FROM duration_shorthands WHERE shorthand = ?1;")?;
                                let duration_iter = stmt.query_map([db], |row| {
                                    let shorthand:String = row.get(0)?;
                                    let duration: i64 = row.get(1)?;

                                    Ok((shorthand, duration))
                                })?;
                                let short_dur = duration_iter
                                    .map(|r| r.map_err(TaskError::from))
                                    .collect::<Result<Vec<_>>>()?;
                                
                                match short_dur.into_iter().next(){
                                    Some((_, duration_secs)) => {
                                        let duration = Duration::seconds(duration_secs);
                                        let now = Local::now().naive_local();
                                        let final_dt = now.add(duration);

                                        Some(final_dt)
                                    }
                                    None => {
                                        return Err(TaskError::Disparity(format!("Shorthand '{}' found in const but not in database.", db)));
                                    }
                                }
                                
                            }
                        }
                        else if let Some(ref dd) = d.due_datetime{
                            let dt = chrono::NaiveDateTime::parse_from_str(dd, "%Y-%m-%d %H:%M:%S")?;
                            Some(dt)
                        }
                        else{
                            None
                        }
                    }
                    None => None,
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

                // match state {
                //     Some(State::Pending) => {

                //     }
                //     Some(State::Doing) => {
                        
                //     }
                //     Some(State::Paused) => {
                        
                //     }
                //     Some(State::Done) => {
                        
                //     }
                //     None => {

                //     }
                // }

                // Convert parsed goal and tag data into Option<String>, insert new string to TAGS and GOALS if needed
                let goal = match goal{
                    Some(g) => {
                        if let Some(ref goal) = g.goal{
                            let g = goal.clone();
                            if !GOALS.with(|goals| goals.borrow().contains(&g)) {
                                return Err(TaskError::InvalidGoal(g));
                            }
                            Some(goal.clone())
                        }
                        // Create new goal
                        else if let Some(ref new_goal) = g.new_goal{
                            GOALS.with(|goals|{
                                goals.borrow_mut().push(new_goal.clone());
                            });
                            conn.execute("INSERT INTO goals (goal_name) VALUES (?1)", params![new_goal])?;
                            Some(new_goal.clone())
                        }
                        else{
                            None
                        }
                    }
                    None => None
                };
                let tag = match tag{
                    Some(t) => {
                        if let Some(ref passed_tag) = t.tag{
                            let t = passed_tag.clone();
                            if !TAGS.with(|tags_| tags_.borrow().contains(&t)) {
                                return Err(TaskError::InvalidTag(t));
                            }
                            Some(t.clone())
                        }
                        // Create new tag
                        else if let Some(ref new_tag) = t.new_tag{
                            TAGS.with(|tags|{
                                tags.borrow_mut().push(new_tag.clone());
                            });
                            conn.execute("INSERT INTO tags (tag_name) VALUES (?1)", params![new_tag])?;
                            Some(new_tag.clone())
                        }
                        else{
                            None
                        }
                    }
                    None => None
                };

                
                // let task = 
                Task::new(title.clone(), priority.clone(), due.clone(), recur_type.clone(), recur_time.clone(), None, state.clone(), goal.clone(), tag.clone())
                //insert_task::in_task(&conn, task)
            }
        }
        _ => return Err(TaskError::WrongWay(String::from("Mode did not match Do, and we are in - fn do_new()")))
        
    };
    new_task
    
    
}