use std::fs;

pub fn get_options(pattern: &str) -> Vec<String> {
    let mut options = vec![];
    let mut current_string = String::from("");
    for c in pattern.chars() {
        if c == '|' {
            options.push(current_string);
            current_string = String::new();
        } else {
            current_string.push(c);
        }
    }
    options.push(current_string);
    options
}

pub fn pattern_splitter(pattern: &str) -> Vec<String> {
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
                let mut pat_push = format!("{{{}", pattern.chars().nth(i+1).unwrap());
                skip_n = 2;
                if pattern.chars().nth(i+2).unwrap() == ',' {
                    skip_n += 1;
                    pat_push.push(',');
                    if pattern.chars().nth(i+3).unwrap() != '}' {
                        skip_n += 1;
                        pat_push.push(pattern.chars().nth(i+3).unwrap());
                    }
                }
                pattern_array.push(pat_push);
            } else {
                pattern_array.push(pattern.chars().nth(i).expect("In string range").to_string())
            }
        }
    }
    return pattern_array;
}

pub fn is_digit(c: char) -> bool {
    let ascii_c = c as u8;
    if ascii_c >= 48 && ascii_c <= 57 {
        return true
    }
    return false
}

pub fn is_alphanumeric(c: char) -> bool {
    let ascii_c = c as u8;
    if ascii_c >= 48 && ascii_c <= 57  ||
       ascii_c >= 65 && ascii_c <= 90  ||
       ascii_c >= 97 && ascii_c <= 122 ||
       ascii_c == 95 {
        return true
    }
    return false
}


pub fn get_files_from_dir(dir: &str) -> Vec<String> {
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
