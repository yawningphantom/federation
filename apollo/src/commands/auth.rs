use crate::commands::*;
use crate::commands::utils::{get_auth, get_user_input};
use std::fs::File;
use console::style;

impl Command for AuthLogin {
    fn run(&self) -> i32 {
        panic!("Not implemented yet!");
    }
}

impl Command for AuthConfigure {
    fn run(&self) -> i32 {
        let auth_token = get_auth();
        if auth_token.is_ok() {
            let prompt = format!("{}", style("You already have an auth token configured. Enter yes to overwrite configuration\n> ")
                .red());
            let should_delete = get_user_input(&prompt, false);
            if should_delete.unwrap().to_lowercase() != "yes" {
                println!("exiting");
                return 0;
            }
        }
        println!("{}", style("Please paste in your API token (visit https://engine-staging.apollographql.com/user-settings to obtain or create an API token)")
            .blue());
        let new_auth_token = get_user_input("ignored", true).unwrap();
        utils::write_auth_token(new_auth_token);
        0
    }
}
