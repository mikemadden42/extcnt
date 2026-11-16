use clap::Parser;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Label printed for files that have no extension.
const NO_EXTENSION_LABEL: &str = "(none)";

/// Files grouped by extension. `None` holds files without an extension and sorts first.
type Groups = BTreeMap<Option<String>, Vec<String>>;

/// Show files grouped by extension
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,

    /// Include hidden files (names starting with '.')
    #[arg(short = 'a', long)]
    all: bool,

    /// Group extensions case-insensitively (e.g. TXT and txt together)
    #[arg(short, long)]
    ignore_case: bool,

    /// Print only the number of files per extension
    #[arg(short, long)]
    count_only: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let groups = match group_by_extension(&args.directory, args.all, args.ignore_case) {
        Ok(groups) => groups,
        Err(err) => {
            eprintln!("Error: {}: {err}", args.directory.display());
            return ExitCode::FAILURE;
        }
    };

    let mut out = BufWriter::new(io::stdout().lock());
    match print_groups(&mut out, &groups, args.count_only).and_then(|()| out.flush()) {
        Ok(()) => ExitCode::SUCCESS,
        // The reader went away (e.g. `extcnt | head`); stop quietly like other CLI tools.
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: writing output: {err}");
            ExitCode::FAILURE
        }
    }
}

fn is_hidden(file_name: &OsStr) -> bool {
    file_name.as_encoded_bytes().starts_with(b".")
}

/// Groups the regular files in `directory` by extension, with each group sorted by name.
///
/// Symlinks are followed: links to files are included, links to directories are not.
/// Entries that cannot be read are reported on stderr and skipped.
fn group_by_extension(
    directory: &Path,
    include_hidden: bool,
    ignore_case: bool,
) -> io::Result<Groups> {
    let mut groups = Groups::new();

    for entry in fs::read_dir(directory)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Warning: skipping entry in {}: {err}", directory.display());
                continue;
            }
        };

        let file_name = entry.file_name();
        if !include_hidden && is_hidden(&file_name) {
            continue;
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path.extension().map(|ext| {
            let ext = ext.to_string_lossy();
            if ignore_case {
                ext.to_lowercase()
            } else {
                ext.into_owned()
            }
        });
        groups
            .entry(extension)
            .or_default()
            .push(file_name.to_string_lossy().into_owned());
    }

    for files in groups.values_mut() {
        files.sort();
    }

    Ok(groups)
}

fn print_groups(out: &mut impl Write, groups: &Groups, count_only: bool) -> io::Result<()> {
    for (extension, files) in groups {
        let label = extension.as_deref().unwrap_or(NO_EXTENSION_LABEL);
        if count_only {
            writeln!(out, "{label}: {}", files.len())?;
            continue;
        }

        writeln!(out, "{label} ({}):", files.len())?;
        for file in files {
            writeln!(out, "- {file}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary directory that is removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("extcnt-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn touch(&self, name: impl AsRef<Path>) {
            fs::write(self.0.join(name), b"").unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn group(extension: Option<&str>, files: &[&str]) -> (Option<String>, Vec<String>) {
        (
            extension.map(str::to_owned),
            files.iter().map(|&f| f.to_owned()).collect(),
        )
    }

    #[test]
    fn groups_and_sorts_files_by_extension() {
        let dir = TempDir::new("groups");
        for name in ["b.txt", "a.txt", "main.rs", "Makefile"] {
            dir.touch(name);
        }
        fs::create_dir(dir.0.join("subdir.d")).unwrap();

        let groups = group_by_extension(&dir.0, false, false).unwrap();

        let expected = Groups::from([
            group(None, &["Makefile"]),
            group(Some("rs"), &["main.rs"]),
            group(Some("txt"), &["a.txt", "b.txt"]),
        ]);
        assert_eq!(groups, expected);
    }

    #[test]
    fn hidden_files_are_skipped_unless_requested() {
        let dir = TempDir::new("hidden");
        dir.touch(".env");
        dir.touch("visible.txt");

        let groups = group_by_extension(&dir.0, false, false).unwrap();
        assert_eq!(groups, Groups::from([group(Some("txt"), &["visible.txt"])]));

        let groups = group_by_extension(&dir.0, true, false).unwrap();
        assert_eq!(
            groups,
            Groups::from([group(None, &[".env"]), group(Some("txt"), &["visible.txt"])])
        );
    }

    #[test]
    fn ignore_case_merges_extensions() {
        let dir = TempDir::new("case");
        dir.touch("upper.TXT");
        dir.touch("lower.txt");

        let groups = group_by_extension(&dir.0, false, false).unwrap();
        assert_eq!(groups.len(), 2);

        let groups = group_by_extension(&dir.0, false, true).unwrap();
        assert_eq!(
            groups,
            Groups::from([group(Some("txt"), &["lower.txt", "upper.TXT"])])
        );
    }

    #[test]
    fn missing_directory_is_an_error() {
        let dir = TempDir::new("missing");
        assert!(group_by_extension(&dir.0.join("nope"), false, false).is_err());
    }

    #[test]
    fn prints_groups_and_counts() {
        let groups = Groups::from([
            group(None, &["Makefile"]),
            group(Some("txt"), &["a.txt", "b.txt"]),
        ]);

        let mut out = Vec::new();
        print_groups(&mut out, &groups, false).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "(none) (1):\n- Makefile\n\ntxt (2):\n- a.txt\n- b.txt\n\n"
        );

        let mut out = Vec::new();
        print_groups(&mut out, &groups, true).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "(none): 1\ntxt: 2\n");
    }

    #[test]
    fn write_errors_are_returned() {
        struct BrokenPipe;

        impl Write for BrokenPipe {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let groups = Groups::from([group(Some("txt"), &["a.txt"])]);
        let err = print_groups(&mut BrokenPipe, &groups, false).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
    }

    // macOS and Windows filesystems reject names that are not valid UTF-8.
    #[cfg(target_os = "linux")]
    #[test]
    fn non_utf8_names_do_not_panic() {
        use std::os::unix::ffi::OsStrExt;

        let dir = TempDir::new("non-utf8");
        dir.touch(OsStr::from_bytes(b"bad\xff.txt"));

        let groups = group_by_extension(&dir.0, false, false).unwrap();
        assert_eq!(
            groups,
            Groups::from([group(Some("txt"), &["bad\u{FFFD}.txt"])])
        );
    }
}
