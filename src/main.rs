use clap::Parser;
use std::fs;

fn main() {
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        #[arg(short, long, default_value = ".")]
        directory: Option<String>,

        #[arg(short, long)]
        is_hidden: Option<bool>,
    }

    let args = Args::parse();
    process_directory(
        &args.directory.unwrap_or_default(),
        args.is_hidden.unwrap_or_default(),
    );
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
