#![allow(dead_code)]
use sqlx::{MySqlPool, FromRow};
use tokio::time::sleep;
use sha256::try_digest;
use md5::compute;
use sha1::{Sha1, Digest};

use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};
use std::io::{self, Read, BufReader, BufRead};
use std::time::Duration;

use crate::model::LogStore;

// use test
use crate::module::test::*;

#[derive(Debug)]
pub struct LogHash {
    device_name: String,
    os_name: String,
    directory: Vec<String>,
    connection: MySqlPool,
}

impl LogHash {
    pub fn new(device_name: String, os_name: String, directory: Vec<String>, connection: MySqlPool) -> Self {
        Self { device_name, os_name, directory, connection }
    }

    async fn sha256sum(file: PathBuf) -> String {
        match try_digest(file.as_path()) {
            Ok(hash) => hash,
            Err(err) => format!("[Failed] {}", err)
        }
    }

    async fn md5sum(file: PathBuf) -> String {
        let mut buffer = Vec::new();
        File::open(file.as_path()).expect("Failed to open file").read_to_end(&mut buffer).expect("Failed to read file");
        format!("{:x}", compute(&buffer))
    }

    async fn sha1sum(file: PathBuf) -> String {
        let mut buffer = Vec::new();
        File::open(file.as_path()).expect("Failed to open file").read_to_end(&mut buffer).expect("Failed to read file");
        let mut sha1 = Sha1::new();
        sha1.update(&buffer);
        format!("{:x}", sha1.finalize())
    }

    // check have for check run client after first time.
    async fn check(&self, dir: String, file: String) -> i32 {
        let query1 = "SELECT device_name, os_name, path, name_file, total_line FROM (SELECT * FROM TB_TR_PDPA_AGENT_LOG0_HASH ORDER BY id DESC) as log0";
        let query2 = format!(
            " WHERE log0.device_name = \"{}\" AND log0.os_name = \"{}\" AND log0.path = \"{}\" AND log0.name_file = \"{}\" LIMIT 1",
            self.device_name,
            self.os_name,
            dir,
            file
        );
        let mut store: Vec<LogStore> = sqlx::query(format!("{}{}",query1,query2).as_str())
            .fetch_all(&self.connection)
            .await.unwrap()
            .into_iter()
            .map(|row| LogStore::from_row(&row).unwrap())
            .collect();

        // Return Here!
        match store.len() {
            0 => -1,
            _ => {
                let selected = store.pop().unwrap();
                selected.total_line.parse::<i32>().unwrap()
            }
        }
    }

    async fn calculate_hash(&mut self, dir:String, file: PathBuf) -> String {
        let line_count = BufReader::new(File::open(file.clone()).unwrap()).lines().count();
        // let res_256= Self::sha256sum(file.clone()).await;
        // let res_md5 = Self::md5sum(file.clone()).await;
        // let res_sha1 = Self::sha1sum(file.clone()).await;
        let filename = file.file_name().unwrap().to_str().unwrap();
        // function on test only!!
        let res_256 = time_function(|| Self::sha256sum(file.clone()), "log0_sha256").await;
        let res_md5 = time_function(|| Self::md5sum(file.clone()), "log0_md5").await;
        let res_sha1 = time_function(|| Self::sha1sum(file.clone()), "log0_sha1").await;
        // Example result := 
        // _host_|||_plaform_ _release_|||_.log_|||_number_|||_sha256_|||_md5_|||_sha1_
        format!("{}|||{}|||{}|||{}|||{}|||{}|||{}|||{}",
            self.device_name,
            self.os_name,
            dir,
            filename,
            line_count,
            res_256,
            res_md5,
            res_sha1
        )
    }

    pub async fn build(&mut self) -> Result<Vec<String>, io::Error> {
        let mut message: Vec<String> = vec![];
        for i in self.directory.clone() {
            let dir_path = Path::new(&i);
            let mut entries: Vec<String> = read_dir(dir_path).unwrap()
                .into_iter()
                .map(|s| {
                    let name = s.unwrap().file_name().to_string_lossy().to_string();
                    if let Some(extension) = dir_path.join(name.clone()).extension().unwrap().to_str() {
                        if extension == "log" || extension == "evtx" {
                            name
                        } else {
                            "Other".to_string()
                        }
                    } else {
                        "None".to_string()
                    }
                })
                .collect();
            // Retain with get true value.
            entries.retain(|x| x != "Other" && x != "None");

            // Read directory first time.
            if entries.is_empty() {
                loop {
                    // Delay wait file in directory.
                    sleep(Duration::from_secs(3)).await;
                    // Call function create 3 type hash.
                    entries = read_dir(dir_path).unwrap()
                        .into_iter()
                        .map(|s| {
                            let name = s.unwrap().file_name().to_string_lossy().to_string();
                            if let Some(extension) = dir_path.join(name.clone()).extension().unwrap().to_str() {
                                if extension == "log" || extension == "evtx" {
                                    name
                                } else {
                                    "Other".to_string()
                                }
                            } else {
                                "None".to_string()
                            }
                        })
                        .collect();
                    match entries.len() {
                        0 => continue,
                        _ => match entries[0].as_str() {
                            "Other" | "None" => continue,
                            _ => break,
                        },
                    }
                }
                // message.push(self.calculate_hash(i.to_string(), dir_path.join(entries[0].as_str())).await);
                // function on test only!!
                message.push(time_function(|| self.calculate_hash(i.to_string(), dir_path.join(entries[0].as_str())), "log0_calculate_hash#1").await);
            } else {
                for j in &entries {
                    // Setup get lines from file.
                    let read_first = BufReader::new(File::open(dir_path.join(j.clone())).unwrap()).lines().count();

                    // Process check file.
                    // if self.check(i.to_string(), j.to_string()).await == -1 {
                    if time_function(|| self.check(i.to_string(), j.to_string()), "log0_check#1").await == -1 {
                        // message.push(self.calculate_hash(i.to_string(), dir_path.join(j)).await);
                        // function on test only!!
                        message.push(time_function(|| self.calculate_hash(i.to_string(), dir_path.join(entries[0].as_str())), "log0_calculate_hash#2").await);
                    } else {
                        // let backup = self.check(i.to_string(), j.to_string()).await;
                        // function on test only!!
                        let backup = time_function(|| self.check(i.to_string(), j.to_string()), "log0_check#2").await;
                        match backup as usize != read_first {
                            true => {
                                // message.push(self.calculate_hash(i.to_string(), dir_path.join(j)).await);
                                // function on test only!!
                                message.push(time_function(|| self.calculate_hash(i.to_string(), dir_path.join(j)), "log0_calculate_hash#3").await);
                            },
                            false => {
                                sleep(Duration::from_secs(2)).await;
                                let read_second = BufReader::new(File::open(dir_path.join(j.clone())).unwrap()).lines().count();
                                if read_first < read_second {
                                    // message.push(self.calculate_hash(i.to_string(), dir_path.join(j)).await);
                                    // function on test only!!
                                    message.push(time_function(|| self.calculate_hash(i.to_string(), dir_path.join(j)), "log0_calculate_hash#4").await);
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }    
            }
        }
        Ok(message)
    }
}
