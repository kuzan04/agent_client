use serde::{Deserialize, Serialize};
use sqlx::{FromRow, mysql::MySqlRow, Row, MySqlPool};
use oracle::{Row as OracleRow, pool::Pool as OraclePool};
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct FilterAgentManage {
    pub code: String,
    pub name: String,
    pub status: i32,
    pub details: String,
    pub token: String,
}

impl FilterAgentManage {
    pub fn new(code: String, name: String, status: i32, details: String, token: String) -> Self {
        Self { code, name, status, details, token }
    }
}

impl Default for FilterAgentManage {
    fn default() -> Self {
        Self { code: "NULL".to_string(), name: "NULL".to_string(), status: -1, details: "NULL".to_string() , token: "NULL".to_string() }
    }
}

impl FromRow<'_, MySqlRow> for FilterAgentManage {
  fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
    let name: String = row.try_get("agm_name")?;
    let code: String = row.try_get("code")?;
    let status: i32 = row.try_get("agm_status")?;
    let details: String = row.try_get("config_detail")?;
    let token: String = row.try_get("agm_token")?;

    Ok(Self { name, code, status, details, token})
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct LogStore {
    pub device_name: String,
    pub os_name: String,
    pub path: String,
    pub name_file: String,
    pub total_line: String,
}

#[allow(dead_code)]
impl LogStore {
    pub fn new(device_name: String, os_name: String, path: String, name_file: String, total_line: String) -> Self {
        Self { device_name, os_name, path, name_file, total_line }
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self { device_name: "NULL".to_string(), os_name: "NULL".to_string(), path: "NULL".to_string(), name_file: "NULL".to_string(), total_line: 0.to_string() }
    }
}

impl FromRow<'_, MySqlRow> for LogStore {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let device_name: String = row.try_get("device_name")?;
        let os_name: String = row.try_get("os_name")?;
        let path: String = row.try_get("path")?;
        let name_file: String = row.try_get("name_file")?;
        let total_line: String = row.try_get("total_line")?;

        Ok(Self { device_name, os_name, path, name_file, total_line })
    }
}

#[derive(Debug)]
pub enum DateOrDateTime {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}

impl ToString for DateOrDateTime {
    fn to_string(&self) -> String {
        match self {
            DateOrDateTime::Date(date) => date.to_string(),
            DateOrDateTime::DateTime(datetime) => datetime.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum PoolRow {
    MyRow(MySqlRow),
    OrRow(OracleRow),
}

#[derive(Debug)]
pub enum PoolType {
    MyPool(MySqlPool),
    OrPool(OraclePool),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SizeFile {
    name: String,
    source: String,
    destination: String,
}

#[allow(dead_code)]
impl SizeFile {
    pub fn new(name: String, source: String, destination: String) -> Self {
        Self { name, source, destination }
    }
}
