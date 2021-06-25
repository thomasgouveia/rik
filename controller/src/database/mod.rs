use rusqlite::{params, Connection, Result};
use std::sync::Arc;

use crate::api::types::tenant::Tenant;

pub struct RickDataBase {
    name: String,
}

#[allow(dead_code)]
impl RickDataBase {
    pub fn new(name: String) -> Arc<RickDataBase> {
        Arc::new(RickDataBase { name })
    }

    pub fn init_tables(&self) -> Result<()> {
        let connection = self.open()?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS tenant (
                id              INTEGER PRIMARY KEY,
                name            TEXT NOT NULL
            )",
            [],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS workload (
                id              INTEGER PRIMARY KEY,
                name            TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn drop_tables(&self) {}

    pub fn open(&self) -> Result<Connection> {
        Ok(Connection::open(format!(
            "./src/database/data/{}.db",
            self.name
        ))?)
    }
}

pub struct RickWorkloadRepository {}
#[allow(dead_code)]
impl RickWorkloadRepository {
    pub fn create_workload(connection: &Connection, name: &str) -> Result<()> {
        connection.execute("INSERT INTO workload (name) VALUES (?1)", params![name])?;
        Ok(())
    }
}

pub struct RickTenantRepository {}
impl RickTenantRepository {
    pub fn create_tenant(connection: &Connection, tenant: &Tenant) -> Result<()> {
        connection.execute(
            "INSERT INTO tenant (name) VALUES (?1)",
            params![tenant.name],
        )?;
        Ok(())
    }

    pub fn find_all_tenant(connection: &Connection) -> Result<Vec<Tenant>> {
        let mut stmt = connection.prepare("SELECT id, name FROM tenant")?;
        let tenants_iter = stmt.query_map([], |row| Ok(Tenant::new(row.get(0)?, row.get(1)?)))?;

        let mut tenants: Vec<Tenant> = Vec::new();
        for tenant in tenants_iter {
            tenants.push(tenant?);
        }
        Ok(tenants)
    }

    pub fn delete_tenants(connection: &Connection, id: usize) -> Result<()> {
        connection.execute("DELETE FROM tenant WHERE id = (?1)", params![id])?;
        Ok(())
    }

    pub fn find_one_tenant(connection: &Connection, id: usize) -> Result<Tenant> {
        let mut stmt = connection.prepare("SELECT id, name FROM tenant WHERE id = (?1)")?;
        match stmt.query_row(params![id], |row| Ok(Tenant::new(row.get(0)?, row.get(1)?))) {
            Ok(tenant) => Ok(tenant),
            Err(err) => Err(err),
        }
    }
}
