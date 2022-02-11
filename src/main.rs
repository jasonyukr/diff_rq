use std::io::{BufReader, BufRead};
use std::process::Stdio;
use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return;
    }

    let mut dir1 = args[1].clone();
    let mut dir2 = args[2].clone();
    if !dir1.ends_with("/") {
        dir1.push('/');
    }
    if !dir2.ends_with("/") {
        dir2.push('/');
    }

    if let Ok(mut child) = Command::new("diff").arg("-rq").arg("--exclude").arg(".git").arg("--exclude").arg(".hg").arg(&dir1).arg(&dir2).stdout(Stdio::piped()).spawn() {
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
//                                println!("[M] \x1b[1;34m{}\x1b[1;0m {} {}", file1_short, file1, file2); // modified: blue
                                println!("[M] \x1b[1;34m{}\x1b[1;0m", file1_short); // modified: blue
                            }
                        }
                    } else if v[0] == "Only" && v[1] == "in" { // Only in PATH: FILE
                        let mut dir_name = v[2].clone();
                        if dir_name.ends_with(":") {
                            dir_name.pop();
                        }
                        let filepath;
                        if dir_name.ends_with("/") {
                            filepath = format!("{}{}", dir_name, v[3]);
                        } else {
                            filepath = format!("{}/{}", dir_name, v[3]);
                        }
                        if filepath.starts_with(&dir1) {
                            println!("[D] \x1b[1;31m{}\x1b[1;0m", filepath); // deleted: red
                        } else if filepath.starts_with(&dir2) {
                            println!("[A] \x1b[1;32m{}\x1b[1;0m", filepath); // added: green
                        } else {
                            // Actually impossible case
                            println!("[U] \x1b[1;35m{}\x1b[1;0m", filepath); // unknown: magenta
                        }
                    }
                }
            }
        }
    }
}
