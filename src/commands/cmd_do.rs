use crate::models::Task;
use crate::errors::{Result, TaskError};
use crate::{cli, GOALS, TAGS};
use rusqlite::Connection;
use crate::DATETIMES;

pub fn do_new(args: cli::JNEL) -> Result<Task> {
    let conv = match &args.mode {
        cli::Mode::Do(cmd) => match &cmd {
            &cli::DoCmd::New { title, priority, due,recurr, state, goal, tag} => {

                // Convert parsed due by data into Option<chrono::Duration> to be passed into Task::new()
                let due = match due {
                    Some(d) => {
                        if let Some(ref db) = d.due_short {
                            if !DATETIMES.with(|dt| dt.borrow().contains(&db)) {
                                return Err(TaskError::InvalidDue(db.clone()));
                            }
                            else{
                                let conn = Connection::open("jnel.sqlite").expect("Unable to open DB");
                                let mut stmt = conn.prepare("SELECT duration FROM duration_shorthands WHERE shorthand = ?1;").unwrap();
                                let duration_iter = stmt.query_map([db], |row| {
                                    Ok(row.get::<_, String>(0)?)
                                }).unwrap();
                                let duration_str = duration_iter.filter_map(rusqlite::Result::ok).next().unwrap();
                                let duration = chrono::Duration::seconds(duration_str.parse::<i64>().unwrap());
                                Some(duration)
                            }
                        }
                        else if let Some(ref dd) = d.due_datetime{
                            let dt = chrono::NaiveDateTime::parse_from_str(dd, "%Y-%m-%d %H:%M:%S")?;
                            let now = chrono::Local::now().naive_local();
                            let duration = dt - now;
                            Some(duration)
                        }
                        else{
                            None
                        }
                    }
                    None => None,
                };

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
                        else if let Some(ref new_goal) = g.new_goal{
                            GOALS.with(|goals|{
                                goals.borrow_mut().push(new_goal.clone());
                            });
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
                        else if let Some(ref new_tag) = t.new_tag{
                            TAGS.with(|tags|{
                                tags.borrow_mut().push(new_tag.clone());
                            });
                            Some(new_tag.clone())
                        }
                        else{
                            None
                        }
                    }
                    None => None
                };

                Task::new(title.clone(), priority.clone(), due.clone(), recurr.clone(), None, state.clone(), goal.clone(), tag.clone())
            }
        }
        _ => return Err(TaskError::WrongWay(String::from("Mode did not match Do, and we are in - fn do_new()")))

    };
    conv
}