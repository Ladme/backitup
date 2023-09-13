// Released under MIT License.
// Copyright (c) 2023 Ladislav Bartos

//! # Back It Up!
//!
//! Stupidly simple crate for backing up files and directories.
//!
//! ## What is this?
//! This crate helps in a common scenario where an application generates an output file that could overwrite another file.
//! Using the `backup` function provided by this crate, you can create backups before any overwriting happens, ensuring the data remains safe.
//! Back It Up! makes sure that even if you write an output file with the same name repeatedly, no data will be lost.
//!
//! ## Usage
//!
//! Run
//!
//! ```bash
//! $ cargo add backitup
//! ```
//!
//! Import the crate in your Rust code:
//!
//! ```rust
//! use backitup::backup;
//! ```
//!
//! ### Creating a Backup
//!
//! To create a backup of a file or directory, use the `backup` function. The function takes the path
//! to the file or directory as an argument and returns the path to the backup file if successful,
//! or an error if the backup operation fails.
//!
//! Note that the content of the file (or directory) is not copied, the file (or directory) is simply **renamed**.
//!
//! ```rust
//! use crate::backitup::backup;
//!
//! let path = "data.txt";
//! match backup(path) {
//!     Ok(backup_path) => println!("Backup created: {:?}", backup_path),
//!     Err(err) => eprintln!("Failed to create backup: {:?}", err),
//! }
//! ```
//!
//! ### Name of the Backup
//! The backup file or directory name is generated based on the original `path`, appending a timestamp
//! in the format "YYYY-MM-DD-HH-MM-SS". If multiple backups are created within the same second, additional
//! information about the microseconds will be appended. The backup name follows the pattern:
//!
//! For files: `"#<parent_directory>/<filename>-<timestamp>(-<microseconds>)#"`
//!
//! For directories: `"#<parent_directory>/<directory_name>-<timestamp>(-<microseconds>)#"`
//!
//! For instance, file `data.txt` backed up on 2023/06/27 at 21:01:13 (local time) will be
//! renamed as `#data.txt-2023-06-27-21-01-13#.
//!
//! ## License
//!
//! This crate is distributed under the terms of the MIT license.
//!

use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use chrono::prelude::*;

/// Creates a backup of the specified file or directory.
/// Returns the path to the backup file if successful, otherwise returns an error.
///
/// # Arguments
///
/// * `path` - The path to the file or directory to be backed up.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `NotFound` - If the specified `path` does not exist.
/// * `Unsupported` - If the `path` is not valid (i.e. not UTF-8, root or ends with '..').
/// * `Io` - If an I/O error occurs during the backup process.
///
/// # Name of the Backup
/// The backup file or directory name is generated based on the original `path`, appending a timestamp
/// in the format "YYYY-MM-DD-HH-MM-SS". If multiple backups are created within the same second, additional
/// information about the microseconds will be appended. The backup name follows the pattern:
///
/// For files: `"#<parent_directory>/<filename>-<timestamp>(-<microseconds>)#"`
///
/// For directories: `"#<parent_directory>/<directory_name>-<timestamp>(-<microseconds>)#"`
///
/// For instance, file `data.txt` backed up on 2023/06/27 at 21:01:13 (local time) will be
/// renamed as `#data.txt-2023-06-27-21-01-13#.
///
/// # Examples
///
/// ```no_run
/// use crate::backitup::backup;
///
/// let path = "data.txt";
/// match backup(path) {
///     Ok(backup_path) => println!("Backup created: {:?}", backup_path),
///     Err(err) => eprintln!("Failed to create backup: {:?}", err),
/// }
/// ```
pub fn backup(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    // check if the path exists
    if !path.as_ref().exists() {
        return Err(Error::new(ErrorKind::NotFound, "Path does not exist."));
    }

    // get the parent directory of the path
    let parent = match path.as_ref().parent() {
        Some(x) => match x.to_str() {
            Some("") => ".",
            Some(x) => x,
            None => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Path is not a valid UTF-8.",
                ))
            }
        },
        None => return Err(Error::new(ErrorKind::Unsupported, "Path is root.")),
    };

    // get the filename from the path
    let filename = match path.as_ref().file_name() {
        Some(x) => match x.to_str() {
            Some(x) => x,
            None => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Path is not a valid UTF-8.",
                ))
            }
        },
        None => return Err(Error::new(ErrorKind::Unsupported, "Path ends in '..'.")),
    };

    // generate the backup file name with a timestamp
    let time = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let mut backup_name = Path::new(&format!("{}/#{}-{}#", parent, filename, &time)).to_path_buf();

    // if a file with the same name already exists, append microseconds
    // repeat until the name of the backup is unique
    while backup_name.exists() {
        let time = Local::now();
        let micros = time.timestamp_subsec_micros();
        let time_fmt = time.format("%Y-%m-%d-%H-%M-%S").to_string();

        backup_name = Path::new(&format!(
            "{}/#{}-{}-{}#",
            parent, filename, &time_fmt, micros
        ))
        .to_path_buf();
    }

    // rename the original file to the backup name
    match fs::rename(path, &backup_name) {
        Ok(()) => Ok(backup_name),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::prelude::*;

    #[test]
    fn file() {
        let mut file = File::create("test_file1.txt").unwrap();
        file.write_all(b"Some content to test.").unwrap();

        let backup = match backup("test_file1.txt") {
            Ok(x) => x,
            Err(_) => panic!("Backup failed."),
        };

        drop(file);

        let mut content = String::new();
        let mut read = File::open(&backup).unwrap();
        read.read_to_string(&mut content).unwrap();

        assert_eq!(content, "Some content to test.");

        fs::remove_file(backup).unwrap();
    }

    #[test]
    fn file_multiple_backups() {
        let mut backups = Vec::new();
        for i in 0..20 {
            let mut file = File::create("test_file2.txt").unwrap();
            let text = format!("Unique string for file {}", i);
            file.write_all(text.as_bytes()).unwrap();

            let backup = match backup("test_file2.txt") {
                Ok(x) => x,
                Err(_) => panic!("Backup failed."),
            };

            backups.push(backup);
        }

        for (i, path) in backups.iter().enumerate() {
            let mut content = String::new();
            let mut read = File::open(&path).unwrap();

            read.read_to_string(&mut content).unwrap();

            let test = format!("Unique string for file {}", i);
            assert_eq!(content, test);

            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn file_in_different_directory() {
        fs::create_dir("test_dir").unwrap();

        let mut file = File::create("test_dir/test_file.txt").unwrap();
        file.write_all(b"Some content to test.").unwrap();

        let backup = match backup("test_dir/test_file.txt") {
            Ok(x) => x,
            Err(_) => panic!("Backup failed."),
        };

        drop(file);

        let mut content = String::new();
        let mut read = File::open(&backup).unwrap();
        read.read_to_string(&mut content).unwrap();

        assert_eq!(content, "Some content to test.");

        fs::remove_file(&backup).unwrap();
        fs::remove_dir("test_dir").unwrap();
    }

    #[test]
    fn file_multiple_backups_in_different_directory() {
        fs::create_dir("test_dir2").unwrap();

        let mut backups = Vec::new();
        for i in 0..20 {
            let mut file = File::create("test_dir2/test_file.txt").unwrap();
            let text = format!("Unique string for file {}", i);
            file.write_all(text.as_bytes()).unwrap();

            let backup = match backup("test_dir2/test_file.txt") {
                Ok(x) => x,
                Err(_) => panic!("Backup failed."),
            };

            backups.push(backup);
        }

        for (i, path) in backups.iter().enumerate() {
            let mut content = String::new();
            let mut read = File::open(&path).unwrap();

            read.read_to_string(&mut content).unwrap();

            let test = format!("Unique string for file {}", i);
            assert_eq!(content, test);

            fs::remove_file(path).unwrap();
        }

        fs::remove_dir("test_dir2").unwrap();
    }

    #[test]
    fn directory() {
        fs::create_dir("test_dir3").unwrap();

        let mut file = File::create("test_dir3/test_file.txt").unwrap();
        file.write_all(b"Some content to test.").unwrap();

        let backup = match backup("test_dir3") {
            Ok(x) => x,
            Err(_) => panic!("Backup failed."),
        };

        drop(file);

        let mut content = String::new();
        let file_in_backup = backup.join(Path::new("test_file.txt"));
        let mut read = File::open(&file_in_backup).unwrap();
        read.read_to_string(&mut content).unwrap();

        assert_eq!(content, "Some content to test.");

        fs::remove_file(&file_in_backup).unwrap();
        fs::remove_dir(&backup).unwrap();
    }

    #[test]
    fn directory_multiple_backups() {
        let mut backups = Vec::new();
        for i in 0..10 {
            fs::create_dir("test_dir4").unwrap();

            let mut file = File::create("test_dir4/test_file.txt").unwrap();
            let text = format!("Unique string for file {}", i);
            file.write_all(text.as_bytes()).unwrap();

            let backup = match backup("test_dir4") {
                Ok(x) => x,
                Err(_) => panic!("Backup failed."),
            };

            backups.push(backup);
        }

        for (i, path) in backups.iter().enumerate() {
            let file_in_backup = path.join(Path::new("test_file.txt"));

            let mut content = String::new();
            let mut read = File::open(&file_in_backup).unwrap();

            read.read_to_string(&mut content).unwrap();

            let test = format!("Unique string for file {}", i);
            assert_eq!(content, test);

            fs::remove_file(&file_in_backup).unwrap();
            fs::remove_dir(&path).unwrap();
        }
    }

    #[test]
    fn nonexistent() {
        match backup("nonexistent.txt") {
            Ok(_) => panic!("Backup should have failed, but it was successful."),
            Err(e) => assert_eq!(e.to_string(), "Path does not exist."),
        };
    }

    #[test]
    fn root() {
        match backup("/") {
            Ok(_) => panic!("Backup should have failed, but it was successful."),
            Err(e) => assert_eq!(e.to_string(), "Path is root."),
        };
    }

    #[test]
    fn empty() {
        match backup("") {
            Ok(_) => panic!("Backup should have failed, but it was successful."),
            Err(e) => assert_eq!(e.to_string(), "Path does not exist."),
        };
    }

    #[test]
    fn dotdot() {
        match backup("..") {
            Ok(_) => panic!("Backup should have failed, but it was successful."),
            Err(e) => assert_eq!(e.to_string(), "Path ends in '..'."),
        };
    }
}
