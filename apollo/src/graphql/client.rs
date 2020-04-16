use serde_json::{Result, Value, Map};
use serde::Serialize;
use std::collections::HashMap;

pub struct ApolloCloudClient {
    endpoint_url: String,
    auth_token: String,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct Query {
    query: String,
}

impl ApolloCloudClient {
    pub fn new(endpoint_url: String, auth_token: String) -> ApolloCloudClient {
        let client = reqwest::blocking::Client::new();
        ApolloCloudClient {
            endpoint_url,
            auth_token,
            client,
        }
    }

    pub fn get_org_memberships(&self) {
        let mut operation_map = HashMap::new();
        operation_map.insert("query", GET_ORG_MEMBERSHIPS_QUERY);
        let res = self.client.post("https://engine-graphql.apollographql.com/api/graphql")
            .json::<HashMap<&str,&str>>(&operation_map).send();
        println!("{:#?}", res);
    }
}

static GET_ORG_MEMBERSHIPS_QUERY: &'static str = "
query GetOrgMemberships {
  me {
    ...on User {
      memberships {
         account {
           id
         }
      }
    }
  }
}
";