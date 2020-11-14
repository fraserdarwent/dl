use std::process;
use std::io::prelude::*;
use std::fs::File;
use std::env;
use regex::Regex;
use flate2::write::GzDecoder;
use tar::Archive;
use reqwest::blocking::get;
fn main() {
    // Functions return the last value in the block

    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: dl <file_url>\nExample commands:\n\tdl https://google.com/index.html\n\tdl https://https://github.com/fluxcd/flux/releases/download/1.21.0/fluxctl_linux_amd64");
        process::exit(1);
    }

    let url = &args[1];
    
    let response =  match get(url) {
        Ok(response) => response,
        Err(error) => {
            println!("Failed to download file");
            if error.to_string().eq("builder error: relative URL without a base"){
                println!("Malformed URL")
            }
            process::exit(1);
        },
    };
    println!("{:#?}",response);
    if response.status().as_u16() != 200 {
        println!("Failed to download file\n{}", response.status());
        process::exit(1);
    }
    
    let file_name_parts: Vec<&str> = url.split("/").collect();
    let file_name = file_name_parts[file_name_parts.len() - 1];
    
    println!("Saving to {}", file_name);
    
    let mut file = match File::create(file_name) {
        Ok(file) => file,
        Err(_) => {
            println!("Failed to create file");
            process::exit(1);
        },
    };

    let bytes = match response.bytes(){
        Ok(bytes) => bytes,
        Err(error) => {
            println!("Failed to decode data {}", error);
            process::exit(1);
        },
    };

    let mut position = 0;

    while position < bytes.len() {
     match file.write(&bytes[position..]){
            Ok(bytes_written) => position += bytes_written,
            Err(_) => {
                println!("Failed to create file");
                process::exit(1);
            },
        };
    }

    // Process file if required
    let re = Regex::new(r"^.*\.tar.gz$").unwrap();
    if re.is_match(file_name){
        println!("Processing .tar.gz");

        let tar_gz = match File::open(file_name){
            Ok(tar_gz) => tar_gz,
            Err(_) => {
                println!("Failed to process file");
                process::exit(1);
            },
        };
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        let folder_name = file_name.replace(".tar.gz", "");
        println!("Extracting to {}", folder_name);
        match archive.unpack(folder_name){
            Ok(_) => {},
            Err(_) => {
                println!("Failed to extract file to folder");
                process::exit(1);
            },
        };
    }
}
