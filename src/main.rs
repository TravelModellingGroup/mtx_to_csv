mod matrix;
mod util;

use rayon::prelude::*;
use std::path::{Path, PathBuf}; // Add rayon for parallel processing

fn main() {
    let command_line_args = std::env::args().collect::<Vec<_>>();
    
    if command_line_args.is_empty() {
        panic!("No command line arguments provided");
    }

    if command_line_args.len() == 1 {
        exit_with_error(&command_line_args[0]);
    }

    let column_output = command_line_args[1].eq_ignore_ascii_case("-c");

    // If column_output is true, we expect at least 3 arguments (program name, -c, and at least one file)
    if column_output && command_line_args.len() < 3 {
        exit_with_error(&command_line_args[0]);
    }

    let files_from_command_line = &command_line_args[(if column_output {2} else { 1})..]; 
    let files = match gather_files(files_from_command_line) {
        Some(files) => files,
        None => {
            eprintln!("No files found");
            std::process::exit(1);
        }
    };

    // Process files in parallel
    files.par_iter().for_each(|path| {
        let result = process_mtx(path, column_output);
        if let Err(e) = result {
            let file_path = path.display();
            eprintln!("Error processing file {file_path}: {e}");
        }
    });
}


fn exit_with_error(program_name : &str) {
    eprintln!("Usage: {program_name} [-c] <input1.mtx(.gz) / *> [<intput2.mtx>]");
    eprintln!(" [-c] is optional and indicates that the output files will be column-oriented.");
    std::process::exit(1);
}

/// Process the mtx file
fn process_mtx(path: &Path, column_output : bool) -> std::io::Result<()> {
    let path = path
        .to_str()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
    let matrix = matrix::Matrix::from_emme_file(path)?;
    let output_path = path.to_string() + ".csv";
    let file = std::fs::File::create(output_path)?;
    // Use a buffered stream
    let mut file = std::io::BufWriter::new(file);
    if column_output {
        matrix.write_csv_column(&mut file)?;
    } else {
        matrix.write_csv_square(&mut file)?;
    }
    Ok(())
}

/// Gather files from command line arguments
/// If the first argument is "*", it will gather all .mtx and .gz files in the current directory.
/// Otherwise, it will gather the files specified in the command line arguments.
/// # Arguments
/// * `files_from_command_line` - A slice of strings containing the command line arguments
/// # Returns
/// * A vector of paths to the files with .mtx or .mtx.gz extensions
fn gather_files(files_from_command_line: &[String]) -> Option<Vec<PathBuf>> {
    let mut files = Vec::new();

    for file in files_from_command_line.iter() 
    {
        if files_from_command_line[0].ends_with("*") {
            // Explore all files with that given directory recursively
            let dir_path = PathBuf::from(file.trim_end_matches("*"));
            println!("Exploring directory recursively: {}", dir_path.display());
            explore_directory_recursive(&dir_path, &mut files);
        }
        else {
            let path = PathBuf::from(file);
            if !path.exists() {
                eprintln!("File {} does not exist", path.display());
            }
            if path.is_dir() {
                explore_directory_recursive(&path, &mut files);
            }
            else {
                files.push(path);
            }
        }       
    }

    Some(files)
}

/// Filter function to check if the file is a .mtx or .mtx.gz file
/// # Arguments
/// * `entry` - A result containing the directory entry
/// # Returns
/// * An optional path to the file if it is a .mtx or .mtx.gz file
/// * None if the file is not a .mtx or .mtx.gz file
fn filter_for_mtx(entry: Result<std::fs::DirEntry, std::io::Error>) -> Option<PathBuf> {
    let entry = entry.ok()?;
    let path =  entry.path();
    if !path.is_file() {
        return None;
    }
    let extension = path.extension()?.to_str()?;
    match extension {
        "mtx" => Some(path),
        "gz" => {
            let stem = path.file_stem()?;
            match util::ends_with(stem, "mtx") {
                true => Some(path),
                false => None,
            }
        }
        _ => None,
    }
}

/// Recursively explore a directory and add all .mtx and .mtx.gz files to the files vector
/// # Arguments
/// * `dir_path` - The directory path to explore
/// * `files` - The vector to add found files to
fn explore_directory_recursive(dir_path: &Path, files: &mut Vec<PathBuf>) {
    let directory = match std::fs::read_dir(dir_path) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error reading directory {}: {e}", dir_path.display());
            return;
        }
    };

    for entry in directory {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading directory entry: {e}");
                continue;
            }
        };

        let path = entry.path();
        
        if path.is_dir() {
            // Recursively explore subdirectories
            explore_directory_recursive(&path, files);
        } else if path.is_file() {
            // Check if it's an .mtx or .mtx.gz file
            if let Some(mtx_file) = filter_for_mtx(Ok(entry)) {
                files.push(mtx_file);
            }
        }
    }
}
