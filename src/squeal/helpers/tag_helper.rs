use std::sync::Arc;
use sqlx::{PgPool, Pool, Postgres, Transaction};
use tokio::sync::mpsc;
use crate::errors::{Result, TaskError};
use crate::cli::{TagArgs};
use crate::{refresh_tags, retrieve_values, DBURL, TAGS};

pub async fn main(mut rx: mpsc::Receiver<Arc<(i32, Option<TagArgs>)>>) -> Result<Vec<i32>>{
    // The pool to clone and pass to the helpers
    let pool = PgPool::connect(&DBURL.lock()?).await?; 

    if let Some(msg) = rx.recv().await{ 

        let handle = tokio::spawn(async move {
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
                        let tag_ids = tags_in(&pool, &new_tags).await;
                        tag_ids
                    }
                    (Some(tags), Some(new_tags)) => { // Case 3: Both was passed
                        
                        let mut tag_ids = tags_in(&pool, &new_tags).await?;

                        let tag_lock = TAGS.lock()?;
                        let tag_ids_frmhash: Vec<i32> = retrieve_values(&tag_lock, &tags).into_iter().copied().collect();
                        tag_ids.extend(tag_ids_frmhash);
                        Ok(tag_ids)
                    }
                    (None, None) => Ok(vec![0])
                },
                None => Ok(vec![0]) // Case 4: No args were passed into the TagArgs
            };
            // ---
            tag_ids

            // Now with the `tag_ids`, store the tag <-> task relations in the table task_tags
            // ---

            
        });
        handle.await?
    }
    else {
        return Err(TaskError::Mysterious(String::from("Dont know")))
    }
}

/// 1. Store the passed vector of tags into the db, 
/// 2. Sync the cache stored tags with the db,
/// 3. Return the ids of the tags stored.
/// 
/// *note: if the tag is already in the db, the fn ignores it and continues*
pub async fn tags_in(pool: &PgPool, new_tags: &Vec<String>) -> Result<Vec<i32>>{
    
    let mut txn: Transaction<'_, Postgres> = pool.begin().await?;
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
    .fetch_all(&mut *txn)
    .await?;
    txn.commit().await?;
    refresh_tags(&pool).await?;
    Ok(new_tag_ids)
}