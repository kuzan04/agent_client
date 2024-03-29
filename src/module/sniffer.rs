use get_if_addrs::{get_if_addrs, IfAddr::V6, IfAddr::V4};
use sqlx::{MySqlPool, Row};
use chrono::Local; use async_ftp::FtpStream;
use syslog::{Facility, Formatter3164};

use std::env;
use std::fs::{
    read_to_string,
    remove_file,
    write,
    metadata,
    create_dir,
    read_dir,
    OpenOptions,
    File,
    DirEntry,
};
use std::io::{
    Read,
    BufReader,
    BufRead,
    BufWriter,
    Write,
    Cursor,
};
use std::process::{
    Command,
    exit,
    Stdio,
};

use crate::model::MyInterface;

// use test
// use crate::module::test::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TaskSniffer {
    connection: MySqlPool,
    interface: String,
    host: String,
    user: String,
    passwd: String,
    mode: i32,
    directory: String,
    status: i32,
}

impl TaskSniffer {
    pub fn new(connection: MySqlPool, details: Vec<String>, interface: String) -> Self {
        let user: String = "ftpuser".to_string();
        let passwd: String = "ftpuser".to_string();
        let mode: i32 = details[0].parse::<i32>().unwrap_or(-1);
        let host: String = match mode {
            0 => format!("{}:{}", env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()), 21),
            1 => format!("{}:{}", env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()), 514), // Tcp: 601, Udp 514.
            _ => {
                println!("[Failed] Unknow mode of sniffer");
                exit(1);
            }
        };
        let directory: String = details[1].to_owned();
        let status = -1;
        Self { connection, interface, host, user, passwd, mode, directory, status }
    }

    async fn set_interface(name: String) -> MyInterface {
        let mut ip = MyInterface::default();
        if let Ok(interfaces) = get_if_addrs() {
            for interface in interfaces {
                if !interface.is_loopback() && interface.name == name && interface.ip().is_ipv4() {
                    ip = match interface.addr {
                        V4(ref value) => MyInterface { 
                            ip: value.ip.to_string(),
                            netmask: value.netmask.to_string(),
                            broadcast: value.broadcast.map(|b| b.to_string()),
                        },
                        V6(ref value) => MyInterface {
                            ip: value.ip.to_string(),
                            netmask: value.netmask.to_string(),
                            broadcast: value.broadcast.map(|b| b.to_string()),
                        },
                    };
                    break
                }
            } 
        }
        ip
    }

    async fn status(&self) -> bool {
        let query = format!("SELECT * FROM TB_TR_PDPA_AGENT_MANAGE WHERE agm_token = \"{}\"", env::var("TOKEN").unwrap());
        let result = sqlx::query(&query).fetch_one(&self.connection).await.unwrap();
        if result.is_empty() {
            remove_file(".env").unwrap();
            println!("[Error] Token expired or has bee deleted");
            exit(1);
        } else {
            match result.try_get::<i32, _>("agm_status") {
                Ok(value) => {
                    let mut env_file_content = read_to_string(".env").unwrap();
                    let new_env = ("STATUS", value.to_string());
                    let current_env = env::var("STATUS").unwrap();
                    env::set_var(new_env.0, new_env.1.clone());
                    env_file_content = env_file_content.replace(&format!("{}=\"{}\"\n", "STATUS", current_env), &format!("{}=\"{}\"\n", new_env.0, new_env.1));
                    write(".env", env_file_content).unwrap();
                    match value {
                        1 => true,
                        0 => false,
                        _ => {
                            println!("[Error] Unknow");
                            exit(1);
                        }
                    }
                },
                Err(err) => {
                    println!("[Error] Can't get status: {}", err);
                    exit(1);
                }
            }
        }
    }

    fn sort(dir_path: &str) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
        let mut list: Vec<_> = read_dir(dir_path).unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        list.sort_by(|a, b| {
            let a_time = a.metadata().unwrap().created().unwrap();
            let b_time = b.metadata().unwrap().created().unwrap();
            a_time.cmp(&b_time)
        });
        Ok(list)
    }

    #[allow(unused_must_use)]
    async fn create_file(ip: String, dir: String, buffer: String) -> String {
        // Setup to create.
        let now = Local::now().format("%H.0,%Y-%m-%d").to_string();
        let set_directory = match dir.chars().nth(dir.chars().count() - 1) {
            Some('/') => dir.clone(),
            _ => format!("{}/", dir.clone()),
        };
        let name = format!("{},{}.snf", ip, now);
        let dir_file = format!("{}{}", set_directory, name);

        // Check directory.
        if !metadata(dir.clone()).unwrap().is_dir() {
            create_dir(dir).unwrap();
        } 

        // Insert to file.
        if metadata(dir_file.clone()).is_ok() {
            // let mut file = OpenOptions::new().append(true).open(dir_file).unwrap();
            let mut file = OpenOptions::new().write(true).append(true).open(dir_file).unwrap();
            // let mut writer = BufWriter::new(file);
            // writer.write_all(buffer.as_bytes()).unwrap();
            writeln!(file, "{}", buffer).unwrap();
        }else{
            let file = File::create(dir_file).unwrap();
            let mut writer = BufWriter::new(file);
            writer.write_all(buffer.as_bytes()).unwrap();
        }

        // List in path and remove old file because new file.
        let mut list_in = Self::sort(&set_directory).unwrap().iter().map(|s| s.file_name().to_string_lossy().to_string()).collect::<Vec<String>>();
        // function on test only!!
        // let mut list_in = time_function(|| Self::sort(&set_directory).unwrap().iter().map(|s| s.file_name().to_string_lossy().to_string()).collect::<Vec<String>>(), "sniffer_sort");
        list_in.retain(|s| !s.contains(".DS_Store"));

        if list_in.len() > 1 {
            for i in list_in.iter().take(list_in.len() - 1) {
                remove_file(format!("{}{}", set_directory, i)).unwrap();
            }
        }

        // Return.
        name
    }

    // Mode 0 (FTP).
    #[allow(unused_must_use)]
    async fn send_ftp(&self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        // Setup varriable to use.
        // local file by default have one file.
        let mut local_file = read_dir(self.directory.clone()).unwrap().map(|s| s.unwrap().file_name().to_string_lossy().to_string()).collect::<Vec<String>>();
        local_file.retain(|s| !s.contains(".DS_Store"));
        let set_dir = &name.clone().split(',').map(|s| s.to_string()).collect::<Vec<String>>()[0];
        
        // Start connect and process another.
        let mut ftp_stream = FtpStream::connect(&self.host).await?;
        ftp_stream.login(&self.user, &self.passwd).await?;

        // List directory in remote and check same client.
        let mut remote_dir = ftp_stream.list(Some("/")).await?
            .iter()
            .map(|entry| {
                let en_len: usize = entry.clone().split_whitespace().map(|e| e.to_string()).collect::<Vec<String>>().len();
                &entry.split_whitespace().map(|e| e.to_string()).collect::<Vec<String>>()[en_len - 1].to_string() == set_dir
            })
            .collect::<Vec<bool>>();
        remote_dir.retain(|&b| b);

        if remote_dir.is_empty() {
            ftp_stream.mkdir(set_dir).await?;
        }

        ftp_stream.cwd(set_dir).await?;

        // (PUT) a file from local to the current working directory of the server.
        // Today (2023-06-10) not check length of file 2 way.
        let mut buffer = Vec::new();
        let full_file = format!("{}{}", self.directory, local_file[0]);
        File::open(full_file).expect("Failed to open file").read_to_end(&mut buffer).expect("Failed to read file");
        let mut content = Cursor::new(buffer);    

        ftp_stream.put(
            &name,
            &mut content
        ).await?;

        // Disconnect FTP server.
        ftp_stream.quit().await;

        // Return.
        Ok(())
    }
    
    // Mode 1 (Syslog).
    async fn send_syslog(&self, event: String) -> Result<(), Box<dyn std::error::Error>> {
        // Setup format log.
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "agent_sniffer".into(),
            pid: 0,
        };

        // Not sure.
        match syslog::tcp(formatter, &format!("{}:{}", self.host, "514")) { 
            // syslog::udp(formatter, "127.0.0.1:5051", &self.host)
            Ok(mut writer) => {
                writer.info(&event)?;
            },
            Err(e) => {
                println!("Failed to connect to syslog server: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        };

        Ok(())
    }

    pub async fn run(&self) {
        let found = Self::set_interface(self.interface.clone()).await;
        // function on test only!!
        // let found = time_function(|| Self::set_interface(self.interface.clone()), "sniffer_set_interface").await;
        match found != MyInterface::default() {
            true => {
                loop {
                    // Core.
                    let mut dump = Command::new("tcpdump")
                        .arg("-i")
                        .arg(&self.interface)
                        .arg("-l")
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to execute command");

                    // Status.
                    let status = self.status().await;
                    // function on test only!!
                    // let status = time_function(|| self.status(), "sniffer_status").await;
                    match status {
                        true => {
                            // Output.
                            let stdout = dump.stdout.take().unwrap();
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().flatten() {
                                // function on test only!!
                                match self.mode {
                                    0 => {
                                        let result = Self::create_file(found.ip.clone(), self.directory.to_owned(), line.clone()).await;
                                        // let result = time_function(|| Self::create_file(found.ip.clone(), self.directory.to_owned(), line.clone()), "sniffer_create_file").await;
                                        // function on test only!!
                                        // match time_function(|| self.send_ftp(result), "sniffer_send_ftp").await {
                                        match self.send_ftp(result).await {
                                            Ok(_) => self.set_history().await,
                                            // function on test only!!
                                            // Ok(_) => {
                                            //     time_function(|| self.set_history(), "sniffer_set_history").await
                                            // },
                                            Err(err) => println!("{:?}", err),
                                        }
                                    },
                                    1 => match self.send_syslog(line).await {
                                    // function on test only!!
                                    // 1 => match time_function(|| self.send_syslog(line), "sniffer_send_syslog").await {
                                        Ok(_) => self.set_history().await,
                                        // function on test only!!
                                        // Ok(_) => time_function(|| self.set_history(), "sniffer_set_history").await,
                                        Err(err) => println!("{:?}", err),
                                    },
                                    _ => {
                                        println!("[Error] Not found mode its");
                                        exit(1);
                                    }
                                }
                            }        
                        },
                        false => dump.kill().unwrap(),
                    }
                }
            },
            false => {
                println!("[Error] Can't interface {} to capture event", self.interface);
                exit(1);
            }
        }
    }

    #[allow(unused_must_use)]
    async fn set_history(&self) {
        let query = format!("SELECT agm_id FROM TB_TR_PDPA_AGENT_MANAGE WHERE agm_token = \"{}\"", env::var("TOKEN").unwrap());
        let selected: i32 = sqlx::query(&query)
            .fetch_one(&self.connection)
            .await.unwrap().get(0);

        let query_select = format!("SELECT * FROM TB_TR_PDPA_AGENT_LISTEN_HISTORY WHERE agm_id = {} GROUP BY agm_id", selected.clone());
        let result_history = match sqlx::query(&query_select)
            .fetch_one(&self.connection)
            .await {
                Ok(res) => res.get::<i64, usize>(1).to_string(),
                Err(_) => "".to_string(),
            };

        match result_history.is_empty() {
            true => sqlx::query(&format!("INSERT INTO TB_TR_PDPA_AGENT_LISTEN_HISTORY (agm_id) VALUE ({})", selected)).execute(&self.connection).await.unwrap(),
            false => sqlx::query(&format!("UPDATE TB_TR_PDPA_AGENT_LISTEN_HISTORY SET _get_ = NOW() WHERE agm_id = {}", selected)).execute(&self.connection).await.unwrap(),
        };
    }
}
