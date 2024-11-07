use rusqlite::{Connection, Result};

pub fn initialize_db(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            description TEXT NOT NULL,
            completed BOOLEAN NOT NULL CHECK (completed IN (0,1)),
            FOREIGN KEY(user_id) REFERENCES users(id)
        )",
        [],
    )?;

    Ok(())
}
