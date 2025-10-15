-- Add migration script here
ALTER TABLE scheduled_tasks ALTER COLUMN priority SET DEFAULT 'Low';