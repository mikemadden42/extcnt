use std::env;
use std::fs;

fn main() {
    let mut include_hidden = false;
    let mut args = env::args().skip(1); // Skip the first argument, which is the program name

    // Check if the --all flag is provided
    if let Some(arg) = args.next() {
        if arg == "--all" {
            include_hidden = true;
        } else {
            // If it's not --all, assume it's a directory and process it
            let directory = arg;
            process_directory(&directory, include_hidden);
            return;
        }
    }

    // If no directory is provided, process the current directory
    process_directory(".", include_hidden);
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
