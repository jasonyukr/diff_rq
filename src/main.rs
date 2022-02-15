use std::io::{BufReader, BufRead};
use std::process::Stdio;
use std::env;
use std::process::Command;
use std::path::Path;

fn print_usage() {
    println!("Usage :");
    println!("  diff_rq dir1 dir2");
    println!("  diff_rq --exclude .git dir1 dir2");
    println!("  diff_rq -uNr --exclude .git dir1 dir2");
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() < 2 {
        print_usage();
        return;
    }

    let mut dir1 = args[args.len()-2].clone();
    let mut dir2 = args[args.len()-1].clone();
    if !dir1.ends_with("/") {
        dir1.push('/');
    }
    if !dir2.ends_with("/") {
        dir2.push('/');
    }

    // We can pass not-existing directory name especially when
    // we run 'diff_rq -uNr' to get added/deleted file list fully
    let mut total_add_mode = false;
    if !Path::new(&dir1).is_dir() {
        total_add_mode = true;
    }
    let mut total_delete_mode = false;
    if !Path::new(&dir2).is_dir() {
        total_delete_mode = true;
    }
    if total_add_mode == true && total_delete_mode == true {
        // At least one directory should exist
        print_usage();
        return;
    }

    if let Ok(mut child) = Command::new("diff").arg("-rq").args(&args).stdout(Stdio::piped()).spawn() {
        if let Some(stdout) = child.stdout.take() {
            for ln in BufReader::new(stdout).lines() {
                if let Ok(line) = ln {
                    let v: Vec<String> = line.split_whitespace().map(str::to_string).collect();
                    if v.len() < 4 {
                        continue;
                    }
                    if v[0] == "Files" && v[2] == "and" { // Files FILE1 and FILE2 differ
                        let file1 = v[1].clone();
                        let file2 = v[3].clone();

                        // Make sure that "diff -rq" output is correct
                        if file1.starts_with(&dir1) && file2.starts_with(&dir2) {
                            let file1_short = file1[dir1.len()..].to_string();
                            let file2_short = file2[dir2.len()..].to_string();
                            if file1_short == file2_short {
                                if total_add_mode {
                                    // "diff_rq -uNr" case from second pahse
                                    println!("A │\x1b[1;32m\u{00a0}\u{25fc}\x1b[1;0m│ \x1b[1;32m{}\x1b[1;0m", file1_short); // added: green
                                } else if total_delete_mode {
                                    // "diff_rq -uNr" case from second phase
                                    println!("D │\x1b[1;31m\u{25fc}\u{00a0}\x1b[1;0m│ \x1b[1;31m{}\x1b[1;0m", file1_short); // deleted: red
                                } else { // normal case
                                    if !Path::new(&file1).is_file() {
                                        // "diff_rq -uNr" case from first phase
                                        println!("A │\x1b[1;32m\u{00a0}\u{25fc}\x1b[1;0m│ \x1b[1;32m{}\x1b[1;0m", file1_short); // added: green
                                    } else if !Path::new(&file2).is_file() {
                                        // "diff_rq -uNr" case from first phase
                                        println!("D │\x1b[1;31m\u{25fc}\u{00a0}\x1b[1;0m│ \x1b[1;31m{}\x1b[1;0m", file1_short); // deleted: red
                                    } else {
                                        // normal case
                                        println!("M │\x1b[1;34m\u{25fc}\u{25fc}\x1b[1;0m│ \x1b[1;34m{}\x1b[1;0m", file1_short); // modified: blue
                                    }
                                }
                                continue;
                            }
                            panic!("Wrong filename");
                        }
                        panic!("Wrong directory prefix");
                    } else if v[0] == "Only" && v[1] == "in" { // Only in PATH: FILE
                        let mut dir_name = v[2].clone();
                        if dir_name.ends_with(":") {
                            dir_name.pop();
                        }
                        let mut filepath;
                        if dir_name.ends_with("/") {
                            filepath = format!("{}{}", dir_name, v[3]);
                        } else {
                            filepath = format!("{}/{}", dir_name, v[3]);
                        }
                        if !filepath.ends_with("/") {
                            if Path::new(&filepath).is_dir() {
                                filepath.push('/');
                            }
                        }
                        if filepath.starts_with(&dir1) {
                            let filepath_short = filepath[dir1.len()..].to_string();
                            println!("D │\x1b[1;31m\u{25fc}\u{00a0}\x1b[1;0m│ \x1b[1;31m{}\x1b[1;0m", filepath_short); // deleted: red
                        } else if filepath.starts_with(&dir2) {
                            let filepath_short = filepath[dir2.len()..].to_string();
                            println!("A │\x1b[1;32m\u{00a0}\u{25fc}\x1b[1;0m│ \x1b[1;32m{}\x1b[1;0m", filepath_short); // added: green
                        } else {
                            panic!("Wrong directory prefix");
                        }
                    }
                }
            }
        }
    }
}
