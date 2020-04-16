use crate::commands::{Command, CreateGraph};
use crate::commands::utils;
use crate::commands::utils::get_auth;
use crate::graphql;

impl Command for CreateGraph {
    fn run(&self) {
        let auth_token = match get_auth() {
            Err(e) => {
                println!("Error authenticating: {}", e);
                return;
            },
            Ok(token) => token,
        };
        let gql_client = graphql::client::ApolloCloudClient::new(
            String::from("https://engine-staging-graphql.apollographql.com"),
            auth_token
        );
        gql_client.get_org_memberships();
        let graph_id = utils::get_user_input("Choose a name for your graph (cannot be changed)").unwrap();
        println!("You have chosen {}. Excellent selection.", graph_id);
    }
}