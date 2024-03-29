extern crate dotenv;

// Package install.
use dotenv::dotenv;
use base64::{Engine as _, engine::general_purpose};
use sqlx::mysql::MySqlPoolOptions;

// Packge already exist after install Rust Language.
use std::fs::{File, metadata, remove_file};
use std::io::{self, Write, BufReader, BufRead};
use std::env;

mod module;
mod model;
use crate::module::handles::Handler;

// on test only!
// use crate::module::test::*;

fn set_env(input: Vec<&str>) {
    let details = vec!["TYPE", "STATUS", "NAME", "HOST", "PORT", "DETAILS", "TOKEN"]
        .iter()
        .zip(input.iter())
        .map(|(&x, &y)| format!("{}=\"{}\"\n", x, y))
        .collect::<Vec<_>>();
    let mut file = File::create(".env").unwrap();
    for i in details {
        file.write_all(i.as_bytes()).unwrap();
    }
    let line_count = BufReader::new(File::open(".env").unwrap()).lines().count();
    if line_count != 7 {
        remove_file(".env").unwrap();
        println!("[Error] Token incorrect!");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    if metadata(".env").is_err() {
        loop {
            print!("Please enter the token you have: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input!");

            match !input.trim().is_empty() && general_purpose::STANDARD.decode(input.trim()).is_ok() {
                true => {
                    if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(input.trim()) {
                        let base = String::from_utf8_lossy(&decoded_bytes).to_string();
                        let mut mix = base.split("&&&").collect::<Vec<&str>>();
                        mix.push(input.trim());
                        // function on test only!!
                        set_env(mix.to_vec());
                        // time_function(|| set_env(mix.to_vec()), "set_env");
                        
                        break 
                    } else {
                        println!("Invalid input. Value must be base64!");
                    }
                },
                false => println!("Invalid input. Please enter the token on base64"),
            }
        }
    }
    
    env::set_var("DB_MAIN", "DOL_PDPA_LOCAL");
    let database_url = format!("mysql://root:P@ssw0rd@localhost:3306/{}", env::var("DB_MAIN").unwrap_or_else(|_| "DOL_PDPA".to_string()));
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await {
            Ok(pool) => {
                pool
            }
            Err(e) => {
                println!("Failed to connect the database: {:?}", e);
                std::process::exit(1);
            }
        };

    // Main process.
    Handler::new(
        pool,
        dotenv::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        dotenv::var("PORT").unwrap_or_else(|_| 5050.to_string()),
    ).task().await;

    // Option send details.
    // dotenv::vars().collect::<HashMap<String, String>>()
}
