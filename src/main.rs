mod modules;
use std::env;
use std::fs::{File};
use std::io::{self, BufRead, BufReader};
use std::process;
use crate::modules::{helpers::get_files_from_dir, matches::matchgen};

fn main() {
    let args_num = env::args().len();
    let mut skip = false;
    let mut pattern = String::from("");
    let mut got_pattern = false;
    let mut files_to_search: Vec<String> = vec![];
    let mut directories_to_search: Vec<String> = vec![];
    let mut found_match = false;
    let mut recursive_search = false;

    for arg_i in 1..args_num {
        if skip {
            skip = false;
            continue;
        }
        if env::args().nth(arg_i).unwrap() == "-E" {
            if args_num == arg_i + 1 {
                println!("Expected a regular expression after '-E'");
                process::exit(1);
            }
            pattern = env::args().nth(arg_i+1).unwrap();
            skip = true;
            got_pattern = true;
        } else if env::args().nth(arg_i).unwrap() == "-r" {
            recursive_search = true;
        } else {
            if recursive_search {
                if files_to_search.len() > 0 {
                    println!("'-r' should go before any directories");
                    process::exit(1);
                }
                directories_to_search.push(env::args().nth(arg_i).unwrap());
            } else {
                files_to_search.push(env::args().nth(arg_i).unwrap());
            }
        }
    }

    if !got_pattern {
        println!("Didn't get a pattern to search ('-E' flag)");
        process::exit(1);
    }
    if recursive_search {
        files_to_search = get_files_from_dir(&directories_to_search[0]);
    }
    
    let mut input_line;
    
    if files_to_search.len() > 0 {
        for filename in &files_to_search {
            let file = File::open(&filename).expect("File should be readable");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                input_line = line.expect("File should split into lines");
                if matchgen(&pattern, &input_line) {
                    if files_to_search.len() > 1 {
                        print!("{}:", filename);
                    }
                    found_match = true;
                    println!("{}", input_line);
                }
            }
        }
    } else {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            input_line = line.expect("Input should split into lines");
            if matchgen(&pattern, &input_line) {
                found_match = true;
                println!("{}", input_line);
            }
        }
    }
    
    if found_match {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
