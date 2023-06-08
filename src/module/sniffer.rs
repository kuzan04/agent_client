use sqlx::MySqlPool;

use std::env;

// use crate::model::SizeFile;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TaskSniffer {
    connection: MySqlPool,
    host: String,
    user: String,
    passwd: String,
    mode: i32,
    directory: Vec<String>,
    timeout: usize,
}

impl TaskSniffer {
    pub fn new(connection: MySqlPool, details: Vec<String>) -> Self {
        let host: String = format!("{}:{}", env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()), 21);
        let user: String = "ftpuser".to_string();
        let passwd: String = "ftpuser".to_string();
        let mode: i32 = details[0].parse::<i32>().unwrap_or(-1);
        let directory: Vec<String> = details[1].split(',').map(|s| s.to_string()).collect();
        let timeout = 120;
        Self { connection, host, user, passwd, mode, directory, timeout }
    }
}
