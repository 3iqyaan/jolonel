use std::sync::Arc;
use sqlx::{PgPool};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use crate::errors::{Result, TaskError};
use crate::cli::{TagArgs};
use crate::squeal::helpers::assemble::{Msg};
use crate::{refresh_tags, retrieve_values, DBURL, TAGS};

pub async fn main(pool: PgPool, mut rx: mpsc::Receiver<Arc<(i32, Option<TagArgs>)>>, tx_asm: mpsc::Sender<Msg>) -> Result<()>{

    let mut join_set: JoinSet<std::result::Result<(), TaskError>> = JoinSet::new();

    while let Some(msg) = rx.recv().await{ 
        println!("tag started");
        let pool_clone = pool.clone();
        
        let tx_clone = tx_asm.clone();
        join_set.spawn(async move {
            
            let (task_id, tags) = &*msg;

            // Parse the TagArgs: if the tags are predefined, obtain their ids; else, create them in the db, sync the cache with the db, and obtain their ids
            // ---
            let tag_ids = match tags{ 
                Some(t) => match (&t.tag, &t.new_tag){
                    (Some(tags), None) => { // Case 1: Passed tags are predefined
                        let tag_lock = TAGS.lock()?;
                        let tag_ids = retrieve_values(&tag_lock, &tags).into_iter().copied().collect();
                        
                        Ok(tag_ids)
                    }
                    (None, Some(new_tags)) => { // Case 2: Create new tags
                        save_tags(&pool_clone, &new_tags).await
                    }
                    (Some(tags), Some(new_tags)) => { // Case 3: Both was passed
                        let mut tag_ids = save_tags(&pool_clone, &new_tags).await?;

                        let tag_lock = TAGS.lock()?;
                        let tag_ids_frmhash: Vec<i32> = retrieve_values(&tag_lock, &tags).into_iter().copied().collect();
                        tag_ids.extend(tag_ids_frmhash);
                        Ok(tag_ids)
                    }
                    (None, None) => Ok(vec![0])
                },
                None => Ok(vec![0]) // Case 4: No args were passed into the TagArgs
            }?;
            // ---
            

            // Now with the `tag_ids`, attach the tags to the task
            // ---
            tx_clone.send(Msg::Tag { task_id: task_id.clone(), tag_ids: tag_ids.clone() }).await.unwrap();
            // ---
            Ok(())
        });
    }
    while let Some(_) = join_set.join_next().await{
        ()
    }
    join_set.abort_all();   
    Ok(())
}

/// 1. Store the passed vector of tags into the db, 
/// 2. Sync the cache stored tags with the db,
/// 3. Return the ids of the tags stored.
/// 
/// *note: if the tag is already in the db, the fn ignores it and continues*
pub async fn save_tags(pool: &PgPool, new_tags: &Vec<String>) -> Result<Vec<i32>>{
    
    let new_tag_ids = sqlx::query_scalar(
        r#"
        WITH input_tags (tag_name) AS (
            SELECT * FROM unnest($1::text[])
        )
        INSERT INTO tags (tag_name)
        SELECT tag_name FROM input_tags
        ON CONFLICT (tag_name) DO NOTHING
        RETURNING id;
        "#
    )
    .bind(new_tags)
    .fetch_all(&*pool)
    .await?;
    refresh_tags(&pool).await?;
    Ok(new_tag_ids)
}