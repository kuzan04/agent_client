use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use sqlx::{FromRow, mysql::MySqlPool};
use sys_info::{hostname, os_release};
use std::{env, fs};
use std::process;

use crate::model::FilterAgentManage;
use crate::module::{
    log0::LogHash,
    // file::DirectoryFile,
    // db::DatabaseCheck,
    // sniffer::TaskSniffer,
};

#[derive(Debug)]
pub struct Handler {
    pub db: MySqlPool,
    pub host: String,
    pub port: String,
}
impl Handler {
    pub fn new(db: MySqlPool, host: String, port: String) -> Handler {
        Handler { db, host, port }
    }

    fn check(fetch: Vec<FilterAgentManage>, token: String) -> FilterAgentManage {
        let mut result = FilterAgentManage::default();
        let mut i = 0;
        while i < fetch.len() {
            if fetch[i].token == token {
                result = fetch[i].clone();
            }
            i+=1
        }
        result
    }

    async fn set_status(fetch: FilterAgentManage, details: FilterAgentManage) -> bool {
        match details.status {
            0 | 1 => {
                let new_env = [
                    ("STATUS", fetch.status.to_string()),
                    ("TOKEN", fetch.token),
                ];
                let mut env_file_content = fs::read_to_string(".env").unwrap();
                for (key, new) in new_env {
                    let current_env = env::var(key).unwrap();
                    env::set_var(key, &new);
                    env_file_content = env_file_content.replace(&format!("{}=\"{}\"\n", key, current_env), &format!("{}=\"{}\"\n", key, new));
                }
                fs::write(".env", env_file_content).unwrap();
                match fetch.status {
                    1 => true,
                    0 => false,
                    _ => {
                        println!("[Error] agent client encountering a problem from database!!!!");
                        process::exit(1);
                    }
                    // Today 2023.06.4 Web alltra function update not have token.
                    // env::set_var("TOKEN", fetch.status);
                }
                
            },
            _ => {
                println!("[Error] Value of status overdue!!!");
                process::exit(1);
            }
        }
    }

    async fn main_task(&mut self) -> Vec<String> {
        match env::var("TYPE") {
            Ok(code) => {
                // Set varriable to send to listener.
                let hostname = match hostname() {
                    Ok(name) => name,
                    Err(_) => "unknow".to_string(),
                };
                let platform = env::consts::OS;
                let release = match os_release() {
                    Ok(os) => os,
                    Err(_) => "unknow".to_string(),
                };
                match code.as_str() {
                    "AG1" => {
                        let details = env::var("DETAILS").unwrap().split(',').map(|s| s.to_string()).collect::<Vec<String>>();
                        match LogHash::new(
                            hostname,
                            format!("{} {}", platform, release),
                            details,
                            self.db.clone()
                        ).build().await {
                                Ok(result) => result,
                                Err(err) => vec![format!("[Failed] {}", err)],
                            }
                    },
                    "AG2" => {
                        println!("2");
                        vec!["Hello".to_string()]
                    },
                    "AG3" => {
                        println!("3");
                        vec!["Hello".to_string()]
                    },
                    "AG4" => {
                        println!("4");
                        vec!["Hello".to_string()]
                    },
                    _ => {
                        println!("[Error] Type of agent client overdue!!!");
                        process::exit(1);
                    }
                }
            },
            Err(err) => {
                println!("[Error] {}", err);
                process::exit(1);
            }
        }
    } 

    pub async fn task(&mut self) {
        loop {
            let mut stream = TcpStream::connect(format!("{}:{}", &self.host, &self.port)).await.expect("Failed to connect to server");
            let query = "SELECT pas.code, am.agm_name, am.config_detail, am.agm_status, am.agm_token FROM TB_TR_PDPA_AGENT_MANAGE as am JOIN TB_TR_PDPA_AGENT_STORE as pas ON am.ags_id = pas.ags_id";
            let result_manager: Vec<FilterAgentManage>= sqlx::query(query)
                .fetch_all(&self.db)
                .await.unwrap()
                .into_iter()
                .map(|row| FilterAgentManage::from_row(&row).unwrap())
                .collect();

            let result_filter = Self::check(result_manager, env::var("TOKEN").unwrap_or_else(|_| "unknow".to_string()));
            let status = match env::var("STATUS").unwrap_or_else(|_| "-1".to_string()).parse::<i32>() {
                Ok(s) => s,
                Err(err) => {
                    let err = format!("[Error] Value of status incorrect!!\n{:?}", err.to_string());
                    println!("{}", err);
                    process::exit(1);
                }
            };
            let reverse_details = FilterAgentManage::new(
                env::var("TYPE").unwrap_or_else(|_| "unknow".to_string()),
                env::var("NAME").unwrap_or_else(|_| "unknow".to_string()),
                status,
                env::var("DETAILS").unwrap_or_else(|_| "unknow".to_string()),
                env::var("TOKEN").unwrap_or_else(|_| "unknow".to_string()),
            );
            // Main task if statement check 2 condition is struct (Object) not default
            // and status client start run.
            if result_filter != FilterAgentManage::default() && Self::set_status(result_filter, reverse_details).await {
                let message: Vec<String> = self.main_task().await
                    .into_iter()
                    .map(|msg| {
                        format!("{}#{}|||{}",
                            env::var("TYPE").unwrap(),
                            env::var("NAME").unwrap(),
                            msg
                        )
                    })
                    .collect();
                for i in message {
                    if let Err(error) = stream.write_all(i.as_bytes()).await {
                        println!("Failed to write to stream: {}", error);
                    }
                }
                // let mut buffer = [0; 1024];
                // match reader.read(&mut buffer).await {
                //     Ok(bytes_read) => {
                //         let get_response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                //         match get_response.as_str() {
                //             "Success" => continue,
                //             _ => continue,
                //         };
                //     },
                //     Err(_) => continue,
                // }
            } else {
                println!("[Error] Check token client for web alltra agent...");
                process::exit(1);
            }
            drop(stream);
        }
        // Separate details to use.
        // let details = self.details.get("DETAILS").unwrap().split(',').map(|s| s.to_string()).collect::<Vec<String>>();

        //
        // Agent Dir/File (AG2)
        // let inform = format!("{}@00{}@{}", self.details.get("TYPE").unwrap(), 1, self.details.get("NAME").unwrap());
        // let msg = format!("{}|||{}|||{}-{}|||{}|||{}|||{}",
        //     self.details.get("NAME").unwrap(),
        //     hostname,
        //     platform,
        //     release,
        //     self.details.get("DETAILS").unwrap(),
        //     "SAMPLE1.xls",
        //     "16384"
        // );
        //
        // drop(stream);
    }
}
