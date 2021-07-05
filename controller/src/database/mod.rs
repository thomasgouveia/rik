use crate::api::types::element::Element;
use dotenv::dotenv;
use rusqlite::{params, Connection, Result};
use std::sync::Arc;

#[allow(dead_code)]
pub struct RikDataBase {
    name: String,
}

#[allow(dead_code)]
impl RikDataBase {
    pub fn new(name: String) -> Arc<RikDataBase> {
        Arc::new(RikDataBase { name })
    }

    pub fn init_tables(&self) -> Result<()> {
        let connection = self.open()?;
        // only work with sqlite for now
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS cluster (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                value           BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS cluster_name_index ON cluster (name);
            CREATE INDEX IF NOT EXISTS cluster_name_id_index ON cluster (name,id);",
        )?;
        Ok(())
    }

    pub fn drop_tables(&self) {}

    pub fn open(&self) -> Result<Connection> {
        dotenv().ok();
        let file_path = match std::env::var("DATABASE_LOCATION") {
            Ok(val) => val,
            Err(_e) => "/var/lib/rik/data/".to_string(),
        };
        std::fs::create_dir_all(&file_path).unwrap();

        let database_path = format!("{}{}.db", file_path, self.name);
        Ok(Connection::open(&database_path)?)
    }
}

pub struct RikRepository {}
impl RikRepository {
    pub fn insert(connection: &Connection, name: &str, value: &str) -> Result<()> {
        connection
            .execute(
                "INSERT INTO cluster (name, value) VALUES (?1, ?2)",
                params![name, value],
            )
            .unwrap();
        Ok(())
    }

    pub fn delete(connection: &Connection, id: usize) -> Result<()> {
        connection.execute("DELETE FROM cluster WHERE id = (?1)", params![id])?;
        Ok(())
    }

    pub fn find_one(connection: &Connection, id: usize, element_type: &str) -> Result<Element> {
        let mut stmt = connection.prepare(&format!(
            "SELECT id, name, value FROM cluster WHERE id = {} AND name LIKE '{}%'",
            id, element_type
        ))?;
        match stmt.query_row([], |row| {
            Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
        }) {
            Ok(element) => Ok(element),
            Err(err) => Err(err),
        }
    }

    pub fn check_duplicate_name(connection: &Connection, name: &str) -> Result<Element> {
        let mut stmt = connection.prepare(&format!(
            "SELECT id, name, value FROM cluster WHERE name LIKE '{}%'",
            name
        ))?;
        match stmt.query_row([], |row| {
            Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
        }) {
            Ok(element) => Ok(element),
            Err(err) => Err(err),
        }
    }

    // TODO: add pagination
    pub fn find_all(connection: &Connection, element_type: &str) -> Result<Vec<Element>> {
        let mut stmt = connection
            .prepare(&format!(
                "SELECT id, name, value FROM cluster WHERE name LIKE '{}%'",
                element_type
            ))
            .unwrap();
        let elements_iter = stmt
            .query_map([], |row| {
                Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap();

        let mut elements: Vec<Element> = Vec::new();
        for element in elements_iter {
            elements.push(element?);
        }
        Ok(elements)
    }

    pub fn update(connection: &Connection, id: usize) -> Result<()> {
        connection.execute(
            "UPDATE cluster SET value=(?1) WHERE id = (?2)",
            params!["Status Updated", id],
        )?;
        Ok(())
    }
}
