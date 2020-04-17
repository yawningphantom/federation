use std::io::{self, Read};
use std::fs::File;
use regex::Regex;
use dirs;
use std::path::Path;
use console::{Term, TermFeatures};

pub fn get_user_input(prompt_string: &str) -> Result<String, std::io::Error> {
    // If we are outputing to a terminal use the stdout for input
    let terminal = Term::stdout();
    let result = if(terminal.features().is_attended()) {
        terminal.write_line("there");
        terminal.read_line_initial_text(prompt_string)?
    } else {
        panic!("This command doesn't work yet without terminal input no pipes :(");
    };
    // Put a empty line
    let trimmed = trim_whitespace(result);
    return Ok(trimmed);
}

pub fn get_auth() -> Result<String,String> {
    let mut file_path = dirs::home_dir().unwrap();
    file_path.push(".apollo/auth-token");
    let mut key_file = match File::open(file_path.as_path()) {
        Ok(file) => file,
        Err(e) => return Err(format!("could not find file {}", file_path.to_str().unwrap())),
    };

    let mut file_contents = String::new();
    key_file.read_to_string(&mut file_contents);

    let trimmed = trim_whitespace(file_contents);
    
    let key_regex = Regex::new(r"^(user|service|internal):([^:]{1,63}):([^:]{1,63})$").unwrap();
    if !key_regex.is_match(&trimmed[..]) {
        return Err(format!("key must match regex {:?}", key_regex.as_str()));
    }

    return Ok(trimmed);
}

fn trim_whitespace(mut input: String) -> String {
    input.truncate(input.trim_end().len());
    input
}
