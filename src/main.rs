use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::process;

const GROUPS: [&str; 2] = ["(", "|"];

fn get_options(pattern: &str) -> (String, String) {
    let mut is_first = true;
    let mut first_string = String::from("");
    let mut second_string = String::from("");
    for c in pattern.chars() {
        if c == '|' {
            is_first = false
        } else {
            if is_first {
                first_string.push(c);
            } else {
                second_string.push(c);
            }
        }
    }
    return (first_string, second_string)
}

fn pattern_splitter(pattern: &str) -> Vec<String> {
    let mut pattern_array: Vec<String> = Vec::new();
    let mut current_patt = String::from("");
    let mut writing = false;
    let mut skip = false;
    let mut in_group = false;
    let mut current_group = String::from("");
    let mut is_alternation = false;
    let mut nest_level = 0;
    let mut skip_n = 0;


    for i in 0..pattern.len() {
        if skip_n > 0 {
            skip_n -= 1;
            continue;
        }
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
        } else if in_group {
            if pattern.chars().nth(i).unwrap() == ')' {
                if nest_level > 0 {
                    nest_level -= 1;
                    current_group.push(pattern.chars().nth(i).unwrap());
                    continue;
                }
                if is_alternation {
                    pattern_array.push('|'.to_string());
                    is_alternation = false;
                } else {
                    pattern_array.push('('.to_string());
                }
                nest_level = 0;
                pattern_array.push(current_group.clone());
                in_group = false;
                current_group = "".to_string();
            } else if pattern.chars().nth(i).unwrap() == '|' && nest_level == 0 {
                is_alternation = true;
                current_group.push(pattern.chars().nth(i).unwrap());
            } else if pattern.chars().nth(i).unwrap() == '(' {
                nest_level += 1;
                current_group.push(pattern.chars().nth(i).unwrap());
            } else {
                current_group.push(pattern.chars().nth(i).unwrap());
            }

        } else {
            if pattern.chars().nth(i).unwrap() == '[' {
                current_patt.push('[');
                writing = true;
            } else if pattern.chars().nth(i).unwrap() == '(' {
                in_group = true;
            } else if pattern.chars().nth(i).unwrap() == '\\' {
                if pattern.chars().nth(i+1).unwrap() == '\\' {
                    pattern_array.push('\\'.to_string());
                } else if is_digit(pattern.chars().nth(i+1).unwrap()) {
                    let mut back_index = String::from("\\");
                    back_index.push(pattern.chars().nth(i+1).unwrap());
                    pattern_array.push(back_index);
                } else {
                    pattern_array.push(pattern[i..i+2].to_string());
                }
                skip = true;
            } else if pattern.chars().nth(i).unwrap() == '{' {
                pattern_array.push(format!("{{{}", pattern.chars().nth(i+1).unwrap()));
                skip_n = 2;
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
    let mut result: bool;
    let regexp: &[String] = &pattern_splitter(regexp_raw);

    // for reg in regexp {
    //     println!("{}", reg);
    // }

    if regexp.len() >= 2 && regexp[0] == "^" {
        (result, _) = matchhere(&regexp[1..], text, &mut [].to_vec(), 0);
    } else {
        loop {
            (result, _) = matchhere(regexp, &text.chars().skip(index).collect::<String>(), &mut [].to_vec(), 0);
            if result || index >= text.len() {
                break;
            }
            index += 1;
        }
    }
    // println!("Matched length: {}", &match_length);
    return result;
}

fn matchhere(regexp: &[String], text: &str, backreferences: &mut Vec<Option<String>>, minimum_length: i32) -> (bool, i32) {

    if regexp.len() == 0 {
        return (true, 0);
    }

    // zero or more group
    if regexp.len() >=3 && GROUPS.contains(&regexp[0].as_str()) && regexp[2] == "*" {
        if regexp.len() == 3 && minimum_length <= 0 {
            return (true, 0)
        } else {
            return matchstargroup(&regexp[0..=1], &regexp[3..], text, minimum_length)
        }

    }

    // exact times
    if regexp.len() >= 2 && regexp[1].chars().nth(0).unwrap_or(' ') == '{' {
        let times = regexp[1].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
        return matchexact(&regexp[0], &regexp[2..], times, text, minimum_length);
    }

    // exact times group
    if regexp.len() >= 3 && GROUPS.contains(&regexp[0].as_str()) && regexp[2].chars().nth(0).unwrap_or(' ') == '{' {
        let times = regexp[2].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
        return matchexactgroup(&regexp[0..=1], &regexp[3..], times, text, minimum_length);
    }

    // backreferences
    if regexp[0].len() > 1 && regexp[0].chars().nth(0).unwrap() == '\\' && is_digit(regexp[0].chars().nth(1).unwrap()) {
        let reference_number = regexp[0].chars().nth(1).unwrap().to_digit(10).unwrap();
        if reference_number > backreferences.len() as u32 {
            return (false, 0);
        }
        let reference_match = backreferences[reference_number as usize - 1].clone();
        if reference_match.is_none() {
            return (false, 0);
        }
        let reference_pattern_array = pattern_splitter(&reference_match.expect("Checked"));
        let (res, index) = matchhere(&reference_pattern_array, text, backreferences, 0);
        if regexp.len() == 1 {
            return (res, index);
        } else {
            let (r, i) = matchhere(&regexp[1..], &text.chars().skip(index as usize).collect::<String>(), backreferences, 0);
            return (r, i + index);
        }
    }

    // match groups
    if regexp[0] == "(" {
        if regexp.len() == 1 {
            return (false, 0)
        }
        let new_reg_array: &[String] = &pattern_splitter(&regexp[1]);
        let mut added_length = 0;
        let backreferences_input_num = backreferences.len();
        while added_length <= text.len() {
            backreferences.push(None);
            let (res, index) = matchhere(new_reg_array, &text, backreferences, added_length as i32);
            if !res {
                return (false, 0);
            }
            
            let ref_match: &str = &text.chars().take(index as usize).collect::<String>();
            backreferences[backreferences_input_num] = Some(ref_match.to_string());

            if regexp.len() == 2{
                if res && index >= minimum_length {
                    return (res, index);
                }
            } else {
                let (r, i) = matchhere(&regexp[2..], &text.chars().skip(index as usize).collect::<String>(), backreferences, minimum_length - index);
                if r {
                    return (r, i + index);
                }
            }
            backreferences.truncate(backreferences_input_num);
            added_length += 1;
        }
        return (false, 0)
    }

    //alternation
    if regexp[0] == "|" {
        if regexp.len() == 1 {
            return (false, 0)
        }
        let (first_string, second_string) = get_options(&regexp[1]);
        let first_reg_array: &[String] = &pattern_splitter(&first_string);
        let second_reg_array: &[String] = &pattern_splitter(&second_string);
        let backreferences_input_num = backreferences.len();
        backreferences.push(None);
        let (res, index) = matchhere(first_reg_array, &text, backreferences, 0);
        if regexp.len() == 2 {
            if res {
                let ref_match: &str = &text.chars().take(index as usize).collect::<String>();
                backreferences[backreferences_input_num] = Some(ref_match.to_string());
                return (res, index);
            } else {
                backreferences.truncate(backreferences_input_num);
                backreferences.push(None);
                let (res, index) = matchhere(second_reg_array, &text, backreferences, 0);
                if res {
                    let ref_match: &str = &text.chars().take(index as usize).collect::<String>();
                    backreferences[backreferences_input_num] = Some(ref_match.to_string());
                    return (res, index);
                }
            }
        } else {
            if res {
                let ref_match: &str = &text.chars().take(index as usize).collect::<String>();
                backreferences[backreferences_input_num] = Some(ref_match.to_string());
                let (r, i) = matchhere(&regexp[2..], &text.chars().skip(index as usize).collect::<String>(), backreferences, 0);
                if r {
                    return (r, i + index)
                }
            } else {
                backreferences.truncate(backreferences_input_num);
                backreferences.push(None);
                let (res, index) = matchhere(second_reg_array, &text, backreferences, 0);
                let ref_match: &str = &text.chars().take(index as usize).collect::<String>();
                backreferences[backreferences_input_num] = Some(ref_match.to_string());
                if res {
                    let (r, i) =  matchhere(&regexp[2..], &text.chars().skip(index as usize).collect::<String>(), backreferences, 0);
                    if r {
                        return (r, i + index);
                    }
                }
            }
        }
        backreferences.truncate(backreferences_input_num);
        return (false, 0);
    }

    //optional
    if regexp.len() >= 2 && regexp[1] == "?" {
        if regexp.len() == 2 {
            return (true, 0);
        } else {
            let (res, pos) = matchhere(&regexp[2..], &text, backreferences, 0);
            if res {
                return (true, pos);
            } else if match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0]) {
                let (res, pos) = matchhere(&regexp[2..], &text.chars().skip(1).collect::<String>(), backreferences, 0);
                if res {
                    return (true, pos + 1);
                }
            }
            return (false, 0);
        }
    }

    // one or more
    if regexp.len() >= 2 && regexp[1] == "+" {
        if regexp.len() == 2 && minimum_length <= 1 {
            let res = match_pattern(&text, &regexp[0]);
            if res {
                return (res, 1)
            }
            return (false, 0)
        } else {
            return matchplus(&regexp[0], &regexp[2..], text, minimum_length)
        }
    }

    // zero or more
    if regexp.len() >= 2 && regexp[1] == "*" {
        if regexp.len() == 2 && minimum_length <= 0 {
            return (true, 0)
        } else {
            return matchstar(&regexp[0], &regexp[2..], text, minimum_length)
        }

    }

    // string end anchor
    if regexp.len() == 1 && regexp[0] == "$" {
        return (text.len() == 0, 0);
    }

    // normal match
    if text.len() > 0 && (match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0])) {
        let (res, leng) = matchhere(&regexp[1..regexp.len()], &text[1..text.len()], backreferences, minimum_length-1);
        return (res, leng + 1);
    }
    return (false, 0);
}

fn matchplus(c: &str, regexp: &[String], text: &str, minimum_length: i32) -> (bool, i32) {
    let mut index = 0;
    while text.len() > index && match_pattern(&text.chars().nth(index).unwrap().to_string(), c) {
        let (res, i) = matchhere(regexp, &text.chars().skip(index+1).collect::<String>(), &mut [].to_vec(), 0);
        let matched_length = i + (index as i32) + 1;
        if res && matched_length >= minimum_length {
            return (true, matched_length);
        }
        index += 1;
    }
    return (false, 0);
}

fn matchexact(c: &str, regexp: &[String], times: i32, text: &str, minimum_length: i32) -> (bool, i32) {
    matchrange(c, regexp, times, times, text, minimum_length)
}

fn matchexactgroup(patt: &[String], regexp: &[String], times: i32, text: &str, minimum_length: i32) -> (bool, i32) {
    matchrangegroup(patt, regexp, times, times, text, minimum_length)
}

fn matchrange(c: &str, regexp: &[String], min_times: i32, max_times: i32, text: &str, minimum_length: i32) -> (bool, i32) {
    let mut index = 0;
    if max_times as usize > text.len() {
        return (false, 0);
    }
    while text.len() > index && index < min_times as usize {
        if !match_pattern(&text.chars().nth(index).unwrap().to_string(), c) {
            return (false, 0);
        }
        index += 1;
    }
    while max_times as usize >= index {
        if index > min_times as usize && !match_pattern(&text.chars().nth(index-1).unwrap().to_string(), c) {
            break;
        }
        let (res, i) = matchhere(regexp, &text.chars().skip(index).collect::<String>(), &mut [].to_vec(), 0);
        let matched_length = i + (index as i32);
        if res && matched_length >= minimum_length {
            return (true, matched_length);
        }
        index += 1;
    }
    return (false, 0);
}

fn matchrangegroup(patt: &[String], regexp: &[String], min_times: i32, max_times: i32, text: &str, minimum_length: i32) -> (bool, i32) {
    let mut index = 0;
    let mut text_matched = 0;
    while text.len() >= text_matched && index < min_times as usize {
        let (res_i, len_i) = matchhere(patt, &text.chars().skip(text_matched).collect::<String>(), &mut vec![], 0);
        if !res_i {
            return (false, 0);
        }
        text_matched += len_i as usize;
        index += 1;
    }
    while text.len() >= text_matched && max_times as usize >= index {
        if index > min_times as usize {
            let (res1, i1) = matchhere(patt, &text.chars().skip(text_matched).collect::<String>(), &mut vec![], 0);
            if res1 {
                text_matched += i1 as usize;
            } else {
                break;
            }
        }
        let (res, i) = matchhere(regexp, &text.chars().skip(text_matched).collect::<String>(), &mut [].to_vec(), 0);
        let matched_length = i + (text_matched as i32);
        if res && matched_length >= minimum_length {
            return (true, matched_length);
        }
        index += 1;
    }
    return (false, 0);
}

fn matchstar(c: &str, regexp: &[String], text: &str, minimum_length: i32) -> (bool, i32) {
    let mut index = 0;
    while text.len() >= index {
        if index > 0 && !match_pattern(&text.chars().nth(index-1).unwrap().to_string(), c) {
            break;
        }
        let (res, i) = matchhere(regexp, &text.chars().skip(index).collect::<String>(), &mut [].to_vec(), 0);
        let matched_length = i + (index as i32);
        if res && matched_length >= minimum_length {
            return (true, matched_length);
        }
        index += 1;
    }
    return (false, 0);
}

fn matchstargroup(patt: &[String], regexp: &[String], text: &str, minimum_length: i32) -> (bool, i32) {
    let mut index = 0;
    let mut text_matched = 0;
    while text.len() > text_matched {
        if index > 0 {
            let (res1, i1) = matchhere(patt, &text.chars().skip(text_matched).collect::<String>(), &mut vec![], 0);
            if res1 {
                text_matched += i1 as usize;
            } else {
                break;
            }
        }
        let (res, i) = matchhere(regexp, &text.chars().skip(text_matched).collect::<String>(), &mut [].to_vec(), 0);
        let matched_length = i + (text_matched as i32);
        if res && matched_length >= minimum_length {
            return (true, matched_length);
        }
        index += 1;
    }
    return (false, 0);
}

fn get_files_from_dir(dir: &str) -> Vec<String> {
    let mut filenames: Vec<String> = vec![];
    let readable_dir = fs::read_dir(dir).expect("should be dir");
    for entry in readable_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            filenames.extend(get_files_from_dir(&path.to_str().unwrap()));
        } else {
            filenames.push(String::from(path.to_str().unwrap()));
        }
    }
    return filenames;
}

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
    
    let mut input_line = String::new();
    
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
        io::stdin().read_line(&mut input_line).unwrap();
        found_match = matchgen(&pattern, &input_line);
        if found_match {
            println!("{}", input_line);
        }
    }
    
    if found_match {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
