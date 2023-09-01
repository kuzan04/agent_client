use async_ftp::FtpStream;
use ssh2::Session;

use std::{io, io::Read};
use std::path::Path;
use std::fs::{File, read_dir, metadata};

// use test
// use crate::module::test::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct DirectoryFile {
    details: Vec<String>,
    directory: Vec<String>,
    ftp_host: String,
    ftp_port: i32,
    ftp_user: String,
    ftp_passwd: String,
}

impl DirectoryFile {
    pub fn new(details: Vec<String>, directory: Vec<String>, ftp_host: String, ftp_port: i32, ftp_user: String, ftp_passwd: String) -> Self {
        Self { details, directory, ftp_host, ftp_port, ftp_user, ftp_passwd }
    }

    #[allow(unused_must_use)]
    async fn ftp_handle(&self, name: String, file: String) {
        match self.ftp_port {
            21 => {
                // Set varriable to use put file ftp.
                let mut buffer = Vec::new();
                File::open(file).expect("Failed to open file").read_to_end(&mut buffer).expect("Failed to read file");
                let mut content = io::Cursor::new(buffer);

                let mut ftp_stream = FtpStream::connect(format!("{}:{}", self.ftp_host, self.ftp_port)).await.unwrap();
                ftp_stream.login(&self.ftp_user, &self.ftp_passwd).await;
                
                let set_name: Vec<&str> = match name.contains('\n') {
                    true => name.split('\n').enumerate().filter(|&(i, _)| i == 1).map(|(_, e)| e).collect::<Vec<&str>>(),
                    false => vec![name.as_str()]
                };
                // Store (PUT) a file from client to server on root directory.
                ftp_stream.put(
                    set_name[set_name.len() - 1],
                    &mut content
                ).await.unwrap();

                // Disconnect FTP server.
                ftp_stream.quit().await;
            },
            22 => {
                // Establish an SSH session.
                let tcp = std::net::TcpStream::connect(format!("{}:{}", self.ftp_host, self.ftp_port)).unwrap();
                let mut sess = Session::new().unwrap();
                sess.set_tcp_stream(tcp);
                sess.handshake().unwrap();

                // Open as SFTP channel.
                let channel = sess.sftp().unwrap();
                
                // Write a local file to the remote server.
                let mut local_file = File::open(file).unwrap();
                let mut remote_file = channel.create(Path::new(&name)).unwrap();
                io::copy(&mut local_file, &mut remote_file).unwrap();

                // Close the SFTP channel and SSH session.
                drop(remote_file);
                drop(channel);
                sess.disconnect(None, "Bye bye", Some("0")).unwrap();
            },
            _ => println!("[Error] unknow type of ftp or sftp"),
        }
    }

    pub async fn build(&self) -> Result<Vec<String>, io::Error> {
        let digits = vec!["0".to_string(); 3];
        let mut message: Vec<String> = vec![];
        for i in 0..self.directory.len() {
            let inform = format!(
                "{}@{}{}@{}@",
                self.details[0], // TYPE of client.
                digits[0..(digits.len() - (i+1).to_string().chars().count())].join(""),
                (i+1),
                self.details[1].split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>().join("_") // NAME of client.
            );
            let mut file: Vec<String> = read_dir(Path::new(&self.directory[i])).unwrap()
                .into_iter()
                .map(|f| match f {
                    Ok(entry) => entry.file_name().to_string_lossy().to_string(),
                    Err(_) => "Err".to_string(),
                })
                .collect();
            file.retain(|x| {
                let ext = Path::new(x).extension().and_then(|ext| ext.to_str()).unwrap_or("");
                x != ".DS_Store" && vec!["log", "xls", "xlsx", "csv", "evtx"].contains(&ext) 
            });
            
            // Process FTP.
            for j in file {
                // Set varriable to use put file ftp.
                let _set_name = format!("{}{}", inform, j.clone());
                let full_file = Path::new(&self.directory[i]).join(j.clone()).to_str().unwrap().to_string();
                let size = match metadata(full_file.clone()) {
                    Ok(l) => l.len(),
                    Err(_) => 0,
                };
                
                // Result
                // self.ftp_handle(set_name, full_file).await;
                self.ftp_handle(j.clone(), full_file).await;
                // function on test only!!
                // time_function(|| self.ftp_handle(set_name, full_file), "file_ftp_handle").await;
                // time_function(|| self.ftp_handle(j.clone(), full_file), "file_ftp_handle").await;
                message.push(
                    format!("{}|||{}|||{}|||{}|||{}",
                        self.details[2], // Device_name
                        self.details[3], // OS-Release
                        self.directory[i],
                        j,
                        size
                    )
                );
            }
        }
        Ok(message)
    }
}
