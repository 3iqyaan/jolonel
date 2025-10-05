use crate::model_squeal::new_task::{goal_in, tag_in};
use crate::models::{Recur, Task};
use crate::errors::{Result, TaskError};
use crate::{cli, GOALS, TAGS};
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use sqlx::{Postgres, Transaction};
use crate::model_squeal::{new_task};
use crate::DATETIMES;

pub async fn do_new(tx: &mut Transaction<'_, Postgres>, args: cli::JNEL) -> Result<Task> {
    let new_task = match &args.mode {
        cli::Mode::Do(cmd) => match &cmd {
            &cli::DoCmd::New { title, priority, due,recur, state, goal, tag} => {

                // Convert parsed due by data into Option<NaiveDateTime> to be passed into Task::new()
                let due_opt = match due{
                    Some(d) => {
                        match (d.due_short.clone(), d.due_datetime.clone()){
                            (Some(shorthand), None) => { // Case 1: Passed shorthand is predefined
                                let datetimes_lock = DATETIMES.lock()?;
                                if datetimes_lock.contains_key(&shorthand) {
                                    let dur_sec = match datetimes_lock.get(&shorthand){
                                        Some(dur) => Ok(*dur),
                                        None => Err(TaskError::InvalidDue(shorthand)),
                                    }?;
                                    let now = Local::now().naive_local();
                                    let dt = now + Duration::seconds(dur_sec);
                                    Ok(Some(dt))
                                }
                                else {
                                    Err(TaskError::InvalidDue(shorthand))
                                }
                            }
                            (None, Some(dt)) => { // Case 2: due was passed as DateTime
                                let dt = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")?;
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

                // Convert parsed goal and tag data into Option<String>, insert new string to TAGS and GOALS if needed
                let goal_opt = match goal{
                    Some(g) => {
                        match (g.goal.clone(), g.new_goal.clone()){
                            (Some(name), None) => { // Case 1: Passed goal is predefined
                                let goals_lock = GOALS.lock()?;
                                if let Some(goal_id) = goals_lock.get(&name) {
                                    Ok(Some(*goal_id))
                                }
                                else {
                                    Err(TaskError::InvalidGoal(name))
                                }
                            }
                            (None, Some(new_goal)) => { // Case 2: Create a new goal
                                goal_in(tx, new_goal).await
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
                                let tags_lock = TAGS.lock()?;
                                if let Some(tag_id) = tags_lock.get(&name){
                                    Ok(Some(*tag_id))
                                }
                                else {
                                    Err(TaskError::InvalidTag(name))
                                }
                            }
                            (None, Some(new_tag)) => { // Case 2: Create a new tag
                                tag_in(tx, new_tag).await
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
    let new_task = new_task::in_task(&mut *tx, new_task).await;
    new_task
    
    
}