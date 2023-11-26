mod decoder;

use crate::decoder::decode_file;
use clap::Parser;
use std::env::set_current_dir;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(about = "Tool to extract RAW data from an image inside a AFF4 container", long_about = None)]
pub struct Args {
    /// Name of the AFF4 object that should extracted
    #[clap(long, short, default_value = "PhysicalMemory")]
    object_name: String,
    /// Either a directory containing AFF4 files or a single AFF4 file
    #[clap(value_parser)]
    path: String,
}

fn main() {
    let args = Args::parse();
    let path = PathBuf::from(&args.path);
    if !path.exists() {
        println!("{} is not a valid file or directory", &args.path);
    }

    if path.is_dir() {
        for entry in fs::read_dir(&path).expect("Failed to read files from directory") {
            let entry = entry.as_ref()
                .expect("Failed to read directory entry");

            let file_name = entry.file_name().to_str()
                .map(|x| x.to_string())
                .expect("Failed to get filename from directory entry");
            if file_name.ends_with("aff4") {
                continue;
            }

            decode_file(&file_name, &args.object_name).expect("Failed to decode AFF4 file");
        }

    } else {
        set_current_dir(
            &path
                .parent()
                .expect("Failed to get parent directory of path"),
        )
        .expect("Failed to set current directory to path");

        let file_name = path
            .file_name()
            .map(|x| x.to_str())
            .flatten()
            .expect("Failed to get filename from path");
        if !file_name.ends_with("aff4") {
            println!("Error: Only AFF4 files are supported");
            exit(0);
        }

        decode_file(file_name, &args.object_name).expect("Failed to decode AFF4 file");
    }
}
