-- Add migration script here

CREATE TYPE task_state AS ENUM ('Scheduled', 'Pending', 'Doing', 'Paused', 'Completed');
CREATE TYPE priority_level AS ENUM ('Low', 'Medium', 'High');


CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    tag_name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS goals (
    id    INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    goal_name  TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id            INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title         TEXT NOT NULL,
    priority      priority_level NOT NULL DEFAULT 'Low',
    due_by        TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    does_recur    BOOLEAN NOT NULL DEFAULT FALSE,
    state         task_state NOT NULL DEFAULT 'Pending',
    goal_id       INTEGER REFERENCES goals(id) ON DELETE SET NULL
);
    

CREATE TABLE IF NOT EXISTS task_tags (
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

CREATE TABLE IF NOT EXISTS task_events (
    id        INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    task_id   INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    state     task_state NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id            INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    does_recur    BOOLEAN NOT NULL DEFAULT TRUE,
    interval      INTEGER, -- in sec
    next_due      TIMESTAMPTZ, -- DateTime as "YYYY-MM-DD HH:MM:SS"
    title         TEXT NOT NULL,
    priority      priority_level NOT NULL,
    due_by        TIMESTAMPTZ, -- DateTime as "YYYY-MM-DD HH:MM:SS" or NULL
    goal_id       INTEGER REFERENCES goals(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduled_task_tags (
    scheduled_task_id   INTEGER NOT NULL REFERENCES scheduled_tasks(id) ON DELETE CASCADE,
    tag_id              INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (scheduled_task_id, tag_id)
);

CREATE TABLE IF NOT EXISTS duration_shorthands (
    id          INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    shorthand   TEXT NOT NULL UNIQUE,
    duration    BIGINT NOT NULL -- duration in seconds
);