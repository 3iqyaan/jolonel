// use crate::cli::{DueArgs, GoalArgs, TagArgs};
// use crate::model_squeal::new_task::{goal_in, in_task, in_task_tag, init_task, tags_in};
// use crate::models::{Priority, Recur, Task};
// use crate::errors::{Result, TaskError};
// use crate::{cli, retrieve_values, DEFAULT_DUE, DEFAULT_TIME, GOALS, TAGS};
// use chrono::{Duration, Local, NaiveDateTime};
// use sqlx::{Postgres, Transaction};
// use crate::DATETIMES;

// impl Task{
//     pub async fn do_new(txn: &mut Transaction<'_, Postgres>, cmd: cli::DoCmd) -> Result<Task> { // Convert the cli parsed structs into squealable stuff
//         match cmd {
//             cli::DoCmd::New { title, priority, due,recur, state, goal, tag} => {
//                 let new_task = Task{
//                     id: init_task(txn, &title).await?,
//                     title: title,
//                     priority: priority.unwrap_or(Priority::Low),
//                     due_by: resolve_due_datetime(due).await?,
//                     recur: recur.unwrap_or(cli::Recurrence{ recur: Some(Recur::None), at_time: Some(DEFAULT_TIME)}).recur.unwrap_or(Recur::None),
//                     at_time: recur.unwrap_or(cli::Recurrence{ recur: Some(Recur::None), at_time: Some(DEFAULT_TIME)}).at_time.unwrap_or(DEFAULT_TIME),
//                     state: state.unwrap_or(crate::models::State::Pending),
//                     goal: resolve_goal_id(txn, goal).await?,
//                     tag: resolve_tag_ids(txn, tag).await?,
                    
//                 };
//                 in_task_tag(txn, &new_task.tag, &new_task.id).await?;
//                 in_task(txn, Ok(new_task)).await
                
//             }
//         }
//     }         
// }

// pub async fn resolve_tag_ids(txn:&mut Transaction<'_, Postgres>, tags: Option<TagArgs>) -> Result<Vec<i32>> {
//     match tags{
//         Some(t) => match (t.tag, t.new_tag){
//             (Some(tags), None) => { // Case 1: Passed tags are predefined
//                 let mut tag_ids = vec![];
//                 let tag_lock = TAGS.lock().unwrap();
//                 for tag in tags{
//                     if let Some(id) = tag_lock.get(&tag){
//                         tag_ids.push(*id);
//                     }
//                 }
//                 crate::errors::Result::Ok(tag_ids)
//             }
//             (None, Some(new_tags)) => { // Case 2: Create new tags
//                 let tag_ids = tags_in(txn, new_tags).await;
//                 tag_ids
//             }
//             (Some(tags), Some(new_tags)) => { // Case 3: Both was passed
                
//                 let mut tag_ids = tags_in(txn, new_tags).await?;

//                 let tag_lock = TAGS.lock().unwrap();
//                 let tag_ids_frmhash: Vec<i32> = retrieve_values(&tag_lock, &tags).into_iter().copied().collect();
//                 tag_ids.extend(tag_ids_frmhash);
//                 Ok(tag_ids)
//             }
//             (None, None) => Ok(vec![0])
//         },
//         None => Ok(vec![0]) // Case 4: No args were passed into the TagArgs
//     }
// }

// pub async fn resolve_goal_id(txn:&mut Transaction<'_, Postgres> ,goal: Option<GoalArgs>) -> Result<i32> {
//     match goal{
//         Some(g) => {
//             match (g.goal, g.new_goal){
//                 (Some(name), None) => { // Case 1: Passed goal is predefined
//                     let goals_lock = GOALS.lock()?;
//                     if let Some(goal_id) = goals_lock.get(&name) {
//                         Ok(*goal_id)
//                     }
//                     else {
//                         Err(TaskError::InvalidGoal(name))
//                     }
//                 }
//                 (None, Some(new_goal)) => { // Case 2: Create a new goal
//                     goal_in(txn, new_goal).await
//                 }
//                 _ => Ok(0), // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
//             }
//         },
//         None => Ok(0) // Case 4: No args were passed into the GoalArgs
//     }
// }

// pub async fn resolve_due_datetime(due: Option<DueArgs>) -> Result<NaiveDateTime> {
//     // Convert parsed due by data into Option<NaiveDateTime> to be passed into Task::new()
//     Ok(match due{
//         Some(d) => {
//             match (d.due_short, d.due_datetime){
//                 (Some(shorthand), None) => { // Case 1: Passed shorthand is predefined
//                     let datetimes_lock = DATETIMES.lock()?;
//                     if datetimes_lock.contains_key(&shorthand) {
//                         let dur_sec = match datetimes_lock.get(&shorthand){
//                             Some(dur) => Ok(*dur),
//                             None => Err(TaskError::InvalidDue(shorthand)),
//                         }?;
//                         let now = Local::now().naive_local();
//                         let dt = now + Duration::seconds(dur_sec);
//                         Ok(Some(dt))
//                     }
//                     else {
//                         Err(TaskError::InvalidDue(shorthand))
//                     }
//                 }
//                 (None, Some(dt)) => { // Case 2: due was passed as DateTime
//                     let dt = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")?;
//                     Ok(Some(dt))
//                 }
//                 _ => Ok(None), // Case 3: Both was passed or neither, both is impossible as we make them a ArgGroup that cannot have multiples, So return None
//             }
//         },
//         None => Ok(None) // Case 4: No args were passed into the DueArgs
//     }?
// .unwrap_or(DEFAULT_DUE))
// }