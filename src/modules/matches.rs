use crate::modules::helpers::{get_options, is_alphanumeric, is_digit, pattern_splitter};
use std::cmp::max;

const GROUPS: [&str; 2] = ["(", "|"];

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

pub fn matchgen(regexp_raw: &str, text: &str) -> Vec<String> {
    let mut index = 0;
    let regexp: &[String] = &pattern_splitter(regexp_raw);
    let mut matches = vec![];

    if regexp.len() >= 2 && regexp[0] == "^" {
        let (result, match_length) = matchhere(&regexp[1..], text, &mut [].to_vec(), 0);
        if result {
            let my_match = text.chars().take(match_length as usize).collect::<String>();
            matches.push(my_match);
        }
    } else {
        loop {
            let (result, match_length) = matchhere(regexp, &text.chars().skip(index).collect::<String>(), &mut [].to_vec(), 0);
            if index >= text.len() {
                break;
            }
            if result {
                let my_match = text.chars().skip(index).take(match_length as usize).collect::<String>();
                matches.push(my_match);
                index += max(match_length - 1, 0) as usize;
            }
            index += 1;
        }
    }
    matches
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

    // range times
    if regexp.len() >= 2 && regexp[1].chars().nth(0).unwrap_or(' ') == '{' {
        if regexp[1].chars().nth(2).unwrap_or(' ') == ',' {
            let times = regexp[1].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
            let max_times;
            if regexp[1].chars().nth(3).unwrap_or(' ') == ' ' {
                max_times = text.len() as i32;
            } else {
                max_times = regexp[1].chars().nth(3).unwrap().to_digit(10).unwrap() as i32;
            }
            println!("maxtimes: {}", max_times);
            return matchrange(&regexp[0], &regexp[2..], times, max_times, text, minimum_length)
        } else {
            let times = regexp[1].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
            return matchexact(&regexp[0], &regexp[2..], times, text, minimum_length);
        }
    }

    // range times group
    if regexp.len() >= 3 && GROUPS.contains(&regexp[0].as_str()) && regexp[2].chars().nth(0).unwrap_or(' ') == '{' {
        if regexp[2].chars().nth(2).unwrap_or(' ') == ',' {
            let times = regexp[2].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
            let max_times;
            if regexp[2].chars().nth(3).unwrap_or(' ') == ' ' {
                max_times = text.len() as i32;
            } else {
                max_times = regexp[2].chars().nth(3).unwrap().to_digit(10).unwrap() as i32;
            }
            return matchrangegroup(&regexp[0..=1], &regexp[3..], times, max_times, text, minimum_length);
        } else {
            let times = regexp[2].chars().nth(1).unwrap_or(' ').to_digit(10).unwrap() as i32;
            return matchexactgroup(&regexp[0..=1], &regexp[3..], times, text, minimum_length);
        }
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
        let options = get_options(&regexp[1]);
        let backreferences_input_num = backreferences.len();
        for option in options {
            let current_reg_array = pattern_splitter(&option);
            backreferences.truncate(backreferences_input_num);
            backreferences.push(None);
            let (res, index) = matchhere(&current_reg_array, text, backreferences, 0);
            if res {
                let ref_match = text.chars().take(index as usize).collect::<String>();
                backreferences[backreferences_input_num] = Some(ref_match);
                if regexp.len() == 2 {
                    return (res, index);
                } else {
                    let (r, i) = matchhere(&regexp[2..], &text.chars().skip(index as usize).collect::<String>(), backreferences, 0);
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
            if text.len() > 0 && match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0]) {
                return (true, 1)
            }
            return (true, 0)
        } else {
            let (res, pos) = matchhere(&regexp[2..], &text, backreferences, 0);
            if text.len() > 0 && match_pattern(&text.chars().nth(0).unwrap().to_string(), &regexp[0]) {
                let (res, pos) = matchhere(&regexp[2..], &text.chars().skip(1).collect::<String>(), backreferences, 0);
                if res {
                    return (true, pos + 1);
                }
            } else if res {
                return (true, pos);
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
    while index < min_times as usize {
        if text.len() <= index || !match_pattern(&text.chars().nth(index).unwrap().to_string(), c) {
            return (false, 0);
        }
        index += 1;
    }
    while text.len() >= index && max_times as usize >= index {
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
    while index < min_times as usize {
        if text.len() < text_matched {
            return (false, 0);
        }
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
