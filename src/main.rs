use std::fs;

fn main() {
    let directory = ".";

    let paths = fs::read_dir(directory)
        .unwrap()
        .map(|res| res.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut files_by_extension = std::collections::BTreeMap::new();

    for path in paths {
        if path.is_file() {
            let extension = match path.extension() {
                Some(os_str) => os_str.to_str().unwrap_or(""),
                None => "",
            };
            files_by_extension
                .entry(extension.to_owned())
                .or_insert(Vec::new())
                .push(path.to_str().unwrap().to_owned());
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
