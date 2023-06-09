use get_if_addrs::{get_if_addrs, IfAddr::V6, IfAddr::V4};
use sqlx::MySqlPool;

use std::env;
use std::io::BufReader;
use std::process::{
    Command,
    exit,
    Stdio,
};

use crate::model::{SizeFile, MyInterface};

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
    timeout: usize,
}

impl TaskSniffer {
    pub fn new(connection: MySqlPool, details: Vec<String>, interface: String) -> Self {
        let host: String = format!("{}:{}", env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()), 21);
        let user: String = "ftpuser".to_string();
        let passwd: String = "ftpuser".to_string();
        let mode: i32 = details[0].parse::<i32>().unwrap_or(-1);
        let directory: String = details[1].to_owned();
        let timeout = 120;
        Self { connection, interface, host, user, passwd, mode, directory, timeout }
    }

    async fn status(&self) -> bool {
        false
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

    pub async fn run(&self) {
        let found = Self::set_interface(self.interface.clone()).await;
        match found != MyInterface::default() {
            true => {
                // Core.
                let mut dump = Command::new("tcpdump")
                    .arg("-i")
                    .arg(&self.interface)
                    .arg("-l")
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to execute command");
                // Input password.
                loop {
                    let stdout = dump.stdout.take().unwrap();
                    let reader = BufReader::new(stdout);
                    println!("{:?}", reader);
                }
            },
            false => {
                println!("[Error] Can't interface {} to capture event", self.interface);
                exit(1);
            }
        }
        // println!("{:?}", )
        // loop {
        // }
    }
}
