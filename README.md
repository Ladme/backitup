# Back It Up!

Stupidly simple crate for backing up files and directories.

## What is this?
This crate helps in a common scenario where an application generates an output file that could overwrite another file.
Using the `backup` function provided by this crate, you can create backups before any overwriting happens, ensuring the data remains safe.
Back It Up! makes sure that even if you write an output file with the same name repeatedly, no data will be lost.

## Usage

Run

```bash
$ cargo add backitup
```

Import the crate in your Rust code:

```rust
use backitup::backup;
```

### Creating a Backup

To create a backup of a file or directory, use the `backup` function. The function takes the path
to the file or directory as an argument and returns the path to the backup file if successful,
or an error if the backup operation fails.

Note that the content of the file (or directory) is not copied, the file (or directory) is simply **renamed**.

```rust
use crate::backitup::backup;

let path = "data.txt";
match backup(path) {
    Ok(backup_path) => println!("Backup created: {:?}", backup_path),
    Err(err) => eprintln!("Failed to create backup: {:?}", err),
}
```

### Name of the Backup
The backup file or directory name is generated based on the original `path`, appending a timestamp
in the format "YYYY-MM-DD-HH-MM-SS". If multiple backups are created within the same second, additional
information about the microseconds will be appended. The backup name follows the pattern:

For files: `"#<parent_directory>/<filename>-<timestamp>(-<microseconds>)#"`

For directories: `"#<parent_directory>/<directory_name>-<timestamp>(-<microseconds>)#"`

For instance, file `data.txt` backed up on 2023/06/27 at 21:01:13 (local time) will be
renamed as `#data.txt-2023-06-27-21-01-13#.

## License

This crate is distributed under the terms of the MIT license.