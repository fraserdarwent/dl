use std::process;
use std::io::prelude::*;
use std::fs::File;
use tokio_compat_02::FutureExt;
use flate2::read::GzDecoder;
use tar::Archive;
use regex::Regex;
use std::env;

#[tokio::main]
async fn main() {
    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: dl <file_url>\nExample commands:\n\tdl https://google.com/index.html\n\tdl https://https://github.com/fluxcd/flux/releases/download/1.21.0/fluxctl_linux_amd64");
        process::exit(1);
    }

    let url = &args[1];
    let file_name_parts: Vec<&str> = url.split("/").collect();
    let file_name = file_name_parts[file_name_parts.len() - 1];

    println!("Downloading your file");

    let mut response = match reqwest::get(url).compat().await {
        Ok(response) => response,
        Err(_) => {
            println!("Unable to download your file");  
            process::exit(1);
        },
    };

    if response.status().as_u16() != 200 {
        println!("There was a problem downloading your file, status code {}", response.status().as_u16());
        process::exit(1);
    }
    
    let mut file = match File::create(file_name) {
        Ok(file) => file,
        Err(_) => {
            println!("There was a problem writing your file to the disk");  
            process::exit(1);
        }
    };

    let mut finished = false;
    
    while !finished {
       match response.chunk().compat().await {
            Ok(bytes) => {
                match bytes {
                    Some(bytes)=>{
                        let write_result = file.write(&bytes);
                        if write_result.is_err() {
                            println!("There was a problem writing your file");
                            process::exit(1);
                        }
                    },
                    None => {
                        finished = true;
                    }
                }
            },
            Err(_) => {
                println!("There was a problem writing your file");  
                process::exit(1);
            }
        };
    }

    drop(file);

    let file = match File::open(file_name) {
        Ok(file) => file,
        Err(_) => {
            println!("There was a problem processing your file");  
            process::exit(1);
        }
    };

    // Process file if required
    let re = Regex::new(r"^.*\.tar.gz$").unwrap();
    if re.is_match(file_name){
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        let folder_name = file_name.replace(".tar.gz", "");
        println!("Auto extracting your file");
        match archive.unpack(folder_name){
            Ok(_) => {},
            Err(error) => {
                println!("{}", error);
                println!("There was a problem extracting your file");
                process::exit(1);
            },
        };
    }
    
    println!("Enjoy");
}
