use std::io::{self, Read};
use std::fs::File;
use regex::Regex;
use dirs;
use std::path::Path;
use console::{Term, TermFeatures};
use std::error::Error;

pub fn get_user_input(prompt_string: &str, secure: bool) -> Result<String, std::io::Error> {
    // If we are outputting to a terminal use the stdout for input
    let result = if atty::is(atty::Stream::Stdout) {
        let terminal = Term::stdout();
        let text = if secure {
            terminal.read_secure_line()?
        } else {
            terminal.read_line_initial_text(prompt_string)?.split_off(prompt_string.len())
        };
        // Write an empty line after hitting enter since terminal will swallow that.
        terminal.write_line("");
        text
    } else {
        println!("{}", prompt_string);
        let mut string_buffer = String::new();
        std::io::stdin().read_line(&mut string_buffer);
        string_buffer
    };
    // Put a empty line
    let trimmed = trim_whitespace(result);
    return Ok(trimmed);
}

pub fn get_auth() -> Result<String, String> {
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

pub fn write_auth_token(token: String) -> Result<(), String> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".apollo");
    std::fs::create_dir_all(&path);
    path.push("auth-token");
    match std::fs::write(path, token) {
        Ok(_) => Ok(()),
        Err (_) => Err(String::from("failed to write token")),
    }
}

fn trim_whitespace(mut input: String) -> String {
    input.truncate(input.trim_end().len());
    input
}
