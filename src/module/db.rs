use sqlx::{MySqlPool, Row};
use oracle::{
    pool::{PoolBuilder, CloseMode},
    Error as OracleError
};
use chrono::{NaiveDate, NaiveDateTime};

use std::env;

use crate::model::{DateOrDateTime, PoolRow, PoolType};

// use test
use crate::module::test::*;

#[derive(Debug)]
pub struct DatabaseCheck {
    connection: MySqlPool,
    _type: i32,
    host: String,
    user: String,
    passwd: String,
    db_name: String,
    tables: Vec<String>,
}

impl DatabaseCheck {
    pub fn new(connection: MySqlPool, _type: i32, mut details: Vec<String>) -> Self {
        let host: String = details.remove(0);
        let user: String = details.remove(0);
        let passwd: String = details.remove(0);
        let db_name: String = details.remove(0);
        let tables: Vec<String> = details[details.len()-1..].to_vec();
        Self { connection, _type, host, user, passwd, db_name, tables }
    }

    fn set_query(_type: i32, column: Vec<String>, query: PoolRow) -> Vec<String> {
        let mut result = Vec::new();
        match query {
            PoolRow::MyRow(query) => {
                for i in column {
                    let value: Result<Option<String>, sqlx::Error> = query.try_get(i.trim());
                    match value {
                        Ok(Some(val)) => result.push(val),
                        Ok(None) => result.push("NULL".to_string()),
                        Err(e) => {
                            let err: Vec<String> = e.to_string().split('`').into_iter().map(|s| s.to_string()).collect();
                            match err[err.len() - 2].as_str() {
                                "INT" | "BIGINT" | "TINYINT" => {
                                    let new_value: i64 = query.get(i.trim()); // int = i32, bigint = i64
                                    result.push(new_value.to_string())
                                },
                                "BOOLEAN" | "BOOL" => {
                                    let new_value: bool = query.get(i.trim());
                                    result.push(new_value.to_string())
                                }
                                "FLOAT" | "DOUBLE" | "DOUBLE PRECISION" | "DECIMAL" | "DEC" | "BIGINT UNSIGNED" => {
                                    let new_value: f64 = query.get(i.trim());
                                    result.push(new_value.to_string())
                                }
                                "DATE" | "DATETIME" | "DATE_TIME" | "TIMESTAMP" => {
                                    let new_value = match query.try_get::<NaiveDate, _>(i.trim()) {
                                        Ok(date) => DateOrDateTime::Date(date),
                                        Err(_) => DateOrDateTime::DateTime(query.try_get::<NaiveDateTime, _>(i.trim()).unwrap()),
                                    };
                                    result.push(new_value.to_string())
                                },
                                _ => result.push("Unknow".to_string()),
                            }
                        },
                    }
                }
            },
            PoolRow::OrRow(query) => {
                for i in 0..column.len() {
                    let value: Result<Option<String>, OracleError> = query.get(i);
                    match value {
                        Ok(Some(val)) => result.push(val),
                        Ok(None) => result.push("NULL".to_string()),
                        Err(e) => {
                            let err: Vec<String> = e.to_string().split('`').into_iter().map(|s| s.to_string()).collect();
                            match err[err.len() - 2].as_str() {
                                "INT" | "BIGINT" | "TINYINT" => {
                                    let new_value: i64 = query.get(i).unwrap(); // int = i32, bigint = i64
                                    result.push(new_value.to_string())
                                },
                                "BOOLEAN" | "BOOL" => {
                                    let new_value: bool = query.get(i).unwrap();
                                    result.push(new_value.to_string())
                                }
                                "FLOAT" | "DOUBLE" | "DOUBLE PRECISION" | "DECIMAL" | "DEC" | "BIGINT UNSIGNED" => {
                                    let new_value: f64 = query.get(i).unwrap();
                                    result.push(new_value.to_string())
                                }
                                "DATE" | "DATETIME" | "DATE_TIME" | "TIMESTAMP" => {
                                    let new_value = match query.get(i) {
                                        Ok(date) => DateOrDateTime::Date(date),
                                        Err(_) => DateOrDateTime::DateTime(query.get(i).unwrap()),
                                    };
                                    result.push(new_value.to_string())
                                },
                                _ => result.push("Unknow".to_string()),
                            }
                        }
                    }
                }
            }
        }
        result
    }

    async fn check_row(old: Vec<String>, current: Vec<String>) -> Option<(Vec<usize>, i32)> {
        // Process check.
        let first_check = old
            .iter()
            .zip(current.iter())
            .map(|(x, y)| {
                x == y
            })
            .collect::<Vec<bool>>();
        match (old.len(), current.len()) {
            (v1, v2) if v1 == v2 => Some((first_check.iter().enumerate().filter(|(_, &value)| !value).map(|(index, _)| index).collect::<Vec<usize>>(), 1)),
            (v1, v2) if v1 > v2 => Some((first_check.iter().enumerate().filter(|(_, &value)| !value).map(|(index, _)| index).collect::<Vec<usize>>(), -1)),
            (v1, v2) if v1 < v2 => Some((first_check.iter().enumerate().filter(|(_, &value)| !value).map(|(index, _)| index).collect::<Vec<usize>>(), 0)),
            _ => None,
        }
    }

    async fn statement(&self, main: Vec<Vec<String>>, old: Vec<String>, current: Vec<String>, columns: Vec<String>, from: String, name: String) -> String {
        // Process check
        // let result_check = Self::check_row(old.clone(), current.clone()).await;
        // function on test only!!
        let result_check = time_function(|| Self::check_row(old.clone(), current.clone()), "database_check_row").await;
        match result_check {
            Some((res, i)) => {
                // Check value in row not equal.
                match res.is_empty() {
                    true => {
                        // self.last(i, main, current, old, columns, vec![from, name]).await
                        // function on test only!!
                        time_function(|| self.last(i, main, current, old, columns, vec![from, name]), "database_last#1").await
                    },
                    false => {
                        // Set Row to Update.
                        let result_id = res.clone()
                            .into_iter()
                            .map(|j| main[j][0].clone())
                            .collect::<Vec<String>>();
                        let result_update = res
                            .into_iter()
                            .map(|j| current[j].clone())
                            .collect::<Vec<String>>();

                        // first update
                        for i in 0..result_id.len() {
                            let convert_value: Vec<String> = result_update[i].replace(['(', ')'], "").split(',').map(|s| s.to_string()).collect();
                            let set_query = convert_value.iter().zip(columns[1..].iter()).map(|(x, y)| format!("{} = {}", y, x)).collect::<Vec<String>>().join(", ");
                            sqlx::query(&format!("UPDATE TB_TR_PDPA_AGENT_DATABASE_CHECK SET {} WHERE {} = {}", set_query, columns[0], result_id[i]))
                                .execute(&self.connection)
                                .await.unwrap();
                        }

                        // self.last(i, main, current, old, columns, vec![from, name]).await
                        // function on test only!!
                        time_function(|| self.last(i, main, current, old, columns, vec![from, name]), "database_last#2").await
                    },
                }
            },
            None => "None".to_string()
        }
    }

    async fn last(&self, status: i32, main: Vec<Vec<String>>, current: Vec<String>, old: Vec<String>, columns: Vec<String>, from_name: Vec<String>) -> String {
        let from = &from_name[0];
        let name = &from_name[1];
        if status == 1 { // Total old equal Total current.
            current.len().to_string()
        } else if status == -1 { // Total old more than row new.
            let select_old = main[current.len()..].to_vec();
            for j in select_old {
                let query_delete = format!("DELETE FROM TB_TR_PDPA_AGENT_DATABASE_CHECK WHERE {} = {}", 
                    columns[0], // id
                    j[0], // on select id.
                );
                sqlx::query(&query_delete)
                    .execute(&self.connection)
                    .await.unwrap();
            }
            current.len().to_string()
        } else if status == 0 { // Total old less total new.
            let set_value = current[old.len()..].
                iter()
                .map(|v| {
                    let mut seprate = v.replace(['(', ')'], "");
                    seprate.push_str(&format!(", \"{}\"", name));
                    format!("({})", seprate)
                })
                .collect::<Vec<String>>()
                .join(", ");
            sqlx::query(&format!("INSERT INTO TB_TR_PDPA_AGENT_DATABASE_CHECK ({}, {}) VALUES {}", columns[1..].join(","), from, set_value))
                .execute(&self.connection)
                .await.unwrap();
            current.len().to_string()
        } else { // Other case.
            "Other".to_string()
        }
    }

    // Mix type db and check pass enum.
    #[allow(clippy::needless_collect, unused_must_use)]
    async fn manage(&self, pool: PoolType, main_id: String, from: String, table: String, columns: Vec<String>) -> String {
        let mut all_columns = columns.clone();
        all_columns.insert(0, main_id);

        // Delete all not client.
        sqlx::query(&format!("DELETE FROM TB_TR_PDPA_AGENT_DATABASE_CHECK WHERE {} IS NULL", from.clone())).execute(&self.connection).await.unwrap();

        // From main.
        let query = format!("SELECT {} FROM TB_TR_PDPA_AGENT_DATABASE_CHECK WHERE {} = '{}' ORDER BY {} ASC",
            all_columns.join(","),
            from,
            table,
            all_columns[0],
        );
        let result_main: Vec<Vec<String>> = sqlx::query(&query).fetch_all(&self.connection).await.unwrap()
            .into_iter()
            .map(|row| {
                let mut result: Vec<String> = vec![];
                for i in &all_columns {
                    result.push(match row.try_get::<String, _>(i.as_str()) {
                        Ok(val) => val,
                        Err(_) => match row.try_get::<i32, _>(i.as_str()) {
                            Ok(val1) => val1.to_string(),
                            Err(_) => "NULL".to_string()
                        },
                    });
                }
                result
            })
            .collect();

        // of client.
        let mut use_table = table.split(':').collect::<Vec<&str>>();
        let use_column = use_table.pop().unwrap().split(',').map(|s| s.to_string()).collect::<Vec<String>>();

        let query_client = format!("SELECT {} FROM {} ORDER BY {} ASC", use_column.join(","), use_table[0], use_column[0]);

        let result_client: Vec<Vec<String>>;
        match pool {
            PoolType::MyPool(pool) => {
                result_client = sqlx::query(&query_client).fetch_all(&pool).await.unwrap()
                    .into_iter()
                    // .map(|row| Self::set_query(self._type, use_column.clone(), PoolRow::MyRow(row)))
                    // function on test only!!
                    .map(|row| time_function(|| Self::set_query(self._type, use_column.clone(), PoolRow::MyRow(row)), "database_set_query#mysql"))
                    .collect();
            },
            PoolType::OrPool(pool) => {
                match pool.get() {
                    Ok(conn) => {
                        result_client = conn.query(&query_client, &[]).unwrap()
                            .into_iter()
                            // .map(|row| Self::set_query(self._type, use_column.clone(), PoolRow::OrRow(row.unwrap())))
                            // function on test only!!
                            .map(|row| time_function(|| Self::set_query(self._type, use_column.clone(), PoolRow::OrRow(row.unwrap())), "database_set_query#oracle"))
                            .collect();
                        conn.close().unwrap();
                    },
                    Err(err) => {
                        println!("[Failed] {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }

        // Main process.
        match result_main.len() {
            0 => {
                // Convert to tuple.
                let value_all: Vec<String> = result_client.clone()
                    .into_iter()
                    .map(|q| {
                        format!("({}, \"{}\")", q.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<String>>().join(","), table)
                    })
                    .collect();
                let query_insert = format!("INSERT INTO TB_TR_PDPA_AGENT_DATABASE_CHECK ({},{}) VALUES {}", columns.join(","), from, value_all.join(", "));
                sqlx::query(&query_insert).execute(&self.connection).await;
                result_client.len().to_string()
            },
            _ => {
                let old_value: Vec<String> = result_main.clone().into_iter().map(|o| {
                    format!("({})", o[1..].iter().map(|v| format!("\'{}\'", v)).collect::<Vec<String>>().join(","))
                }).collect();
                let new_value: Vec<String> = result_client.clone().into_iter().map(|o| {
                    format!("({})", o.iter().map(|v| format!("\'{}\'", v)).collect::<Vec<String>>().join(","))
                }).collect();
                // self.statement(result_main, old_value, new_value, all_columns, from, table).await
                // function on test only!!
                time_function(|| self.statement(result_main, old_value, new_value, all_columns, from, table), "database_statement").await
            }
        }
    }
    
    #[allow(unused_must_use)]
    pub async fn build(&self) -> Result<Vec<String>, std::io::Error> {
        let mut message = vec![];
        let query = format!(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = 'TB_TR_PDPA_AGENT_DATABASE_CHECK' AND TABLE_SCHEMA = '{}' ORDER BY ORDINAL_POSITION",
            env::var("DB_MAIN").unwrap_or_else(|_| "DOL_PDPA".to_string())
        );
        let mut all_field: Vec<String> = sqlx::query(&query)
            .fetch_all(&self.connection)
            .await.unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("COLUMN_NAME"))
            .collect();

        // Not sure.
        all_field.pop();
        let column_id = all_field.remove(0);
        let from_client = all_field.pop().unwrap();

        for i in &self.tables {
            let size_field = i.split(',').count();
            let select_field = &all_field[0..size_field];
            match self._type {
                1 => {
                    match MySqlPool::connect(&format!("mysql://{}:{}@{}:{}/{}", self.user, self.passwd, self.host, 3306, self.db_name)).await {
                        Ok(pool) => {
                            // let result = format!("{}|||{}", 
                            //     i,
                            //     self.manage(PoolType::MyPool(
                            //         pool.clone()),
                            //         column_id.to_owned(),
                            //         from_client.to_owned(),
                            //         i.to_string(),
                            //         select_field.to_vec()
                            //     ).await
                            // );
                            // function on test only!!
                            let result = format!("{}|||{}", 
                                i,
                                time_function(||
                                    self.manage(PoolType::MyPool(
                                        pool.clone()),
                                        column_id.to_owned(),
                                        from_client.to_owned(),
                                        i.to_string(),
                                        select_field.to_vec()
                                    ),
                                    "database_manage#mysql"
                                ).await
                            );
                            pool.close();
                            message.push(result);
                        },
                        Err(err) => {
                            println!("[Failed] {}", err);
                            std::process::exit(1);
                        },
                    };
                },
                0 => {
                    match PoolBuilder::new(&self.user, &self.passwd, &format!("//{}:{}/{}", self.host, 1521, self.db_name)).max_connections(10).build() {
                        Ok(pool) => {
                            // let result = format!("{}|||{}", 
                            //     i,
                            //     self.manage(PoolType::OrPool(
                            //         pool.clone()),
                            //         column_id.to_owned(),
                            //         from_client.to_owned(),
                            //         i.to_string(),
                            //         select_field.to_vec()
                            //     ).await
                            // );
                            // function on test only!!
                            let result = format!("{}|||{}",
                                i,
                                time_function(|| self.manage(PoolType::OrPool(
                                    pool.clone()),
                                    column_id.to_owned(),
                                    from_client.to_owned(),
                                    i.to_string(),
                                    select_field.to_vec()
                                    ),
                                    "database_manage#oracle"
                                ).await
                            );
                            pool.close(&CloseMode::Default);
                            message.push(result);
                        },
                        Err(err) => {
                            println!("[Failed] {}", err);
                            std::process::exit(1);
                        },
                    }
                },
                _ => {
                    println!("[Error] Type database not support");
                    std::process::exit(1);
                }
            }
        }
        Ok(message)
    }
}
