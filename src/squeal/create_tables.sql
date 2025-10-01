BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS tags (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS goals (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_name  TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    title         TEXT NOT NULL,
    priority      TEXT,
    due_by      TEXT,
    recurrence    TEXT, -- "Daily", "Weekly", etc. or NULL
    state         TEXT NOT NULL -- "NotStarted", "Started", "Paused", "Resumed", "Completed"
);

ALTER TABLE tasks ADD COLUMN goal_id INTEGER
    REFERENCES goals(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS task_tags (
    task_id INTEGER NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)  REFERENCES tags(id)  ON DELETE CASCADE
);

CREATE TABLE task_events (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id   INTEGER NOT NULL,
    state     TEXT NOT NULL, -- "NotStarted", "Started", "Paused", "Resumed", "Completed"
    timestamp TEXT NOT NULL,

    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS duration_shorthands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shorthand TEXT NOT NULL UNIQUE,
    duration INTEGER NOT NULL -- duration in seconds
)

COMMIT;