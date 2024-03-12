use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("File Extension List")
        .version("0.1.0")
        .author("Michael Madden")
        .about("Lists files by extension")
        .arg(
            Arg::with_name("directory")
                .help("Sets the directory to list files")
                .index(1),
        )
        .arg(
            Arg::with_name("all")
                .short('a')
                .long("all")
                .help("Include hidden files"),
        )
        .get_matches();

    let directory = matches.value_of("directory").unwrap_or(".");
    let include_hidden = matches.is_present("all");

    process_directory(directory, include_hidden);
}

fn process_directory(directory: &str, include_hidden: bool) {
    let paths = fs::read_dir(directory)
        .unwrap()
        .map(|res| res.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut files_by_extension = std::collections::BTreeMap::new();

    for path in paths {
        // Skip hidden files if the flag is not provided
        if !include_hidden {
            if let Some(file_name) = path.file_name() {
                if let Some(name) = file_name.to_str() {
                    if name.starts_with('.') {
                        continue;
                    }
                }
            }
        }

        if path.is_file() {
            let extension = match path.extension() {
                Some(os_str) => os_str.to_str().unwrap_or(""),
                None => "",
            };
            // Use PathBuf's file_name() method to get the file name without './' prefix
            let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
            files_by_extension
                .entry(extension.to_owned())
                .or_insert(Vec::new())
                .push(file_name);
        }
    }

    for (extension, files) in files_by_extension {
        println!("{extension}:");
        let mut files = files;
        files.sort();
        for file in files {
            println!("- {file}");
        }
        println!();
    }
}
