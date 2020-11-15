use std::process;
use std::io::prelude::*;
use std::fs::File;
use tokio_compat_02::FutureExt;
use flate2::read::GzDecoder;
use tar::Archive;
use regex::Regex;
use std::env;
use std::io::{Error, ErrorKind};
use std::io;
use std::fs;
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
    if Regex::new(r"^.*\.tar.gz$").unwrap().is_match(file_name){
        println!("Auto extracting your file");

        // Read the tar gzip and write into a regular folder
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        let folder_name = file_name.trim_end_matches(".tar.gz");
        match archive.unpack(folder_name){
            Ok(_) => {},
            Err(error) => {
                println!("{}", error);
                println!("There was a problem extracting your file");
                process::exit(1);
            },
        };
    } else if Regex::new(r"^.*\.zip$").unwrap().is_match(file_name){
        println!("Auto extracting your file");
        match unzip(file_name, file_name.trim_end_matches(".zip")){
            Ok(_) => {},
            Err(error) => {
                println!("{}", error);
                println!("There was a problem extracting your file");
                process::exit(1);
            },
        };
       
    };
    println!("Enjoy");
}


fn unzip(source: &str, destination:&str) -> std::io::Result<()>{
    // Read the zip and write into a regular folder
    let archive_file = match File::open(source){
        Ok(file) => file,
        Err(_) => {
          return Err(Error::new(ErrorKind::Other, "A problem occured opening the source file"));
        }
    };
    let mut archive = zip::ZipArchive::new(archive_file).unwrap();

    // Create the root folder
    match fs::create_dir_all(destination) {
        Ok(_) => {},
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "A problem occured creating a destination folder"));
        }
    }

    // Extract the archive into the root folder
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).expect("A problem occured iterating over the files in the archive");
        if file.name().ends_with("/") {
            // File is a folder
            match fs::create_dir_all(format!("{0}/{1}", destination, file.name())) {
                Ok(_) => {},
                Err(_) => {
                    return Err(Error::new(ErrorKind::Other, "A problem occured creating a destination folder"));
                }
            }
        } else {
            // File is a file
            let mut destination_file = match File::create(format!("{0}/{1}", destination, file.name())) {
                Ok(file) => file,
                Err(error) => {
                    println!("{}", error);
                    return Err(Error::new(ErrorKind::Other, "A problem occured creating the destination file"));
                }
            };

            match io::copy(&mut file, &mut destination_file){
                Ok(_) => {},
                Err(_) => {
                    return Err(Error::new(ErrorKind::Other, "A problem occured extracting source file"));
                }
            }
        }
    };

    // This is to appease the return type of result
    Ok(())
}
