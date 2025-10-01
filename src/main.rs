use std::env;
use std::io;
use std::process;

fn pattern_splitter(pattern: &str) -> Vec<String> {
    let mut pattern_array: Vec<String> = Vec::new();
    let mut current_patt = String::from("");
    let mut writing = false;
    let mut skip = false;


    for i in 0..pattern.len() {
        if skip {
            skip = false;
            continue;
        }
        if writing {
            current_patt.push(pattern.chars().nth(i).expect("In string range"));
            if pattern.chars().nth(i).unwrap() == ']' {
                writing = false;
                pattern_array.push(current_patt);
                current_patt = "".to_string();
            } 
        } else {
            if pattern.chars().nth(i).unwrap() == '[' {
                // println!("{}", i);
                current_patt.push('[');
                writing = true;
            } else if pattern.chars().nth(i).unwrap() == '\\' {
                if pattern.chars().nth(i+1).unwrap() == '\\' {
                    pattern_array.push('\\'.to_string());
                } else {
                    pattern_array.push(pattern[i..i+2].to_string());
                }
                skip = true;
            } else {
                pattern_array.push(pattern.chars().nth(i).expect("In string range").to_string())
            }
        }
    }
    return pattern_array;
}

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
        if pattern == "." && input_line.len() > 0 {
            return true;
        }
        return input_line.contains(pattern);
    } else if pattern.contains("\\d") {
        return input_line.chars().any(|c| is_digit(c));
    } else if pattern.contains("\\w") {
        return input_line.chars().any(|c| is_alphanumeric(c));
    } else if pattern.chars().nth(0) == Some('[') && pattern.chars().nth(pattern.len()-1) == Some(']') {
        if pattern.chars().nth(1) == Some('^') {
            // Negative
            let unmatchables = &pattern[2..(pattern.len()-1)];
            for c in input_line[0..(input_line.len())].chars() {
                let mut matched = false;
                for unmatchable in unmatchables.chars() {
                    if c == unmatchable {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return true;
                }
            }
            return false;
        } else {
            // Positive
            let matchables = &pattern[1..(pattern.len()-1)];
            for matchable in matchables.chars() {
                if input_line.contains(matchable) {
                    return true;
                }
            }
            return false;
        }
    } else {
        panic!("Unhandled pattern: {}", pattern);
    }
}

fn matchgen(regexp_raw: &str, text: &str) -> bool {
    let mut index = 0;
    let mut matched_length: i32;
    let mut result: bool;
    let regexp: &[String] = &pattern_splitter(regexp_raw);

    if regexp.len() >= 2 && regexp[0] == "^" {
        (result, matched_length) = matchhere(&regexp[1..], text);
    } else {
        loop {
            (result, matched_length) = matchhere(regexp, &text.chars().skip(index).collect::<String>());
            if result || index >= text.len() {
                break;
            }
            index += 1;
        }
    }
    println!("Matched length: {}", matched_length);
    return result;
}

fn matchhere(regexp: &[String], text: &str) -> (bool, i32) {
    if regexp.len() == 0 {
        return (true, 0);
    }

    if regexp.len() >= 2 && regexp[1] == "?" {
        if regexp.len() == 2 {
            return (true, 0);
        } else {
            let (res, pos) = matchhere(&regexp[2..], &text);
            println!("{}", pos);
            if res {
                return (true, pos);
            } else if match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0]) {
                let (res, pos) = matchhere(&regexp[2..], &text.chars().skip(1).collect::<String>());
                if res {
                    return (true, pos + 1);
                }
            }
            return (false, 0);
        }
    }

    if regexp.len() >= 2 && regexp[1] == "+" {
        if regexp.len() == 2 {
            return (match_pattern(&text, &regexp[0]), 0)
        } else {
            return matchplus(&regexp[0], &regexp[2..], text)
        }
    }

    if regexp.len() == 1 && regexp[0] == "$" {
        return (text.len() == 0, 0);
    }

    if text.len() > 0 && (match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0])) {
        let (res, leng) = matchhere(&regexp[1..regexp.len()], &text[1..text.len()]);
        return (res, leng + 1);
    }
    return (false, 0);
}

fn matchplus(c: &str, regexp: &[String], text: &str) -> (bool, i32) {
    let mut index = 0;
    while text.len() > index + 1 && match_pattern(&text.chars().nth(index).unwrap().to_string(), c) {
        let (res, i) = matchhere(regexp, &text.chars().skip(index+1).collect::<String>());
        if res {
            return (true, i + (index as i32) + 1);
        }
        index += 1;
    }
    return (false, 0);
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // println!("pat length: {}", pattern_array.len());
    // for pat in &pattern_array {
    //     println!("{}", pat);
    // }

    if matchgen(&pattern, &input_line) {
        println!("Pattern found!");
        process::exit(0)
    } else {
        println!("Pattern not found :(");
        process::exit(1)
    }
}
