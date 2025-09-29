use std::env;
use std::io;
use std::process;

fn is_digit(c: char) -> bool {
    let ascii_c = c as u8;
    if ascii_c >= 48 && ascii_c <= 57 {
        return true
    }
    return false
}

fn is_alphanumeric(c: char) -> bool {
    let ascii_c = c as u8;
    if ascii_c >= 48 && ascii_c <= 57  ||
       ascii_c >= 65 && ascii_c <= 90  ||
       ascii_c >= 97 && ascii_c <= 122 ||
       ascii_c == 95 {
        return true
    }
    return false
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {

    if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else if pattern.contains("\\d") {
        return input_line.chars().any(|c| is_digit(c));
    } else if pattern.contains("\\w") {
        return input_line.chars().any(|c| is_alphanumeric(c));
    } else if pattern.chars().nth(0) == Some('[') && pattern.chars().nth(pattern.len()-1) == Some(']') {
        let matchables = &pattern[1..(pattern.len()-1)];
        for matchable in matchables.chars() {
            if input_line.contains(matchable) {
                return true;
            }
        }
        return false;
    } else {
        panic!("Unhandled pattern: {}", pattern);
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    //eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        println!("Pattern found!");
        process::exit(0)
    } else {
        println!("Pattern not found :(");
        process::exit(1)
    }
}
