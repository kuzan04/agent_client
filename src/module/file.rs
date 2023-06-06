use ftp::FtpStream;
use ssh2::{Session, Channel};

use std::{io, io::Read};
use std::path::Path;
use std::fs::{File, read_dir, metadata};

#[derive(Debug)]
#[allow(dead_code)]
pub struct DirectoryFile {
    code: String,
    name: String,
    device_name: String,
    os_name: String,
    directory: Vec<String>,
    ftp_host: String,
    ftp_port: i32,
    ftp_user: String,
    ftp_passwd: String,
}

impl DirectoryFile {
    pub fn new(code: String, name: String, device_name: String, os_name: String, directory: Vec<String>, ftp_host: String, ftp_port: i32, ftp_user: String, ftp_passwd: String) -> Self {
        Self { code, name, device_name, os_name, directory, ftp_host, ftp_port, ftp_user, ftp_passwd }
    }

    async fn ftp_handle(&self, name: String, file: String) {
        match self.ftp_port {
            21 => {
                // Set varriable to use put file ftp.
                let mut buffer = Vec::new();
                File::open(file).expect("Failed to open file").read_to_end(&mut buffer).expect("Failed to read file");
                let mut content = io::Cursor::new(String::from_utf8(buffer).expect("Invalid UTF-8 sequence").as_bytes());

                let mut ftp_stream = FtpStream::connect(format!("{}:{}", self.ftp_host, self.ftp_port)).unwrap();
                // Store (PUT) a file from client to server on root directory.
                ftp_stream.put(name.as_str(), &mut content);
            },
            22 => {
                // Establish an SSH session.
                let tcp = std::net::TcpStream::connect(format!("{}:{}", self.ftp_host, self.ftp_port)).unwrap();
                let mut sess = Session::new().unwrap();
                sess.set_tcp_stream(tcp);
                sess.handshake().unwrap();

                // Open as SFTP channel.
                let mut channel = sess.sftp().unwrap();
                
                // Write a local file to the remote server.
                let mut local_file = File::open(file).unwrap();
                let mut remote_file = channel.create(name).unwrap();
                io::copy(&mut local_file, &mut remote_file).unwrap();

                // Close the SFTP channel and SSH session.
                drop(remote_file);
                drop(channel);
                sess.disconnect(None, "Bye bye", Some(0)).unwrap();
            },
            _ => println!("[Error] unknow type of ftp or sftp"),
        }
    }

    pub async fn build(&self) -> Result<Vec<String>, io::Error> {
        let digits = vec!["0"; 10];
        let mut message: Vec<String> = vec![];
        for i in 0..self.directory.len() {
            let inform = format!(
                "{}@{}@{}@",
                self.code,
                digits[0..i].join(""),
                self.name
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
                let set_name = format!("{}{}", inform, j.clone());
                let full_file = Path::new(&self.directory[i]).join(j.clone()).to_str().unwrap().to_string();
                let size = match metadata(full_file.clone()) {
                    Ok(l) => l.len(),
                    Err(_) => 0,
                };
                
                // Result
                self.ftp_handle(set_name, full_file).await;
                message.push(
                    format!("{}|||{}|||{}|||{}|||{}|||{}",
                        self.device_name,
                        self.os_name,
                        self.directory[i],
                        j,
                        size,
                        inform
                    )
                );
            }
        }
        Ok(message)
    }
}
