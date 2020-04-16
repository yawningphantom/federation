use serde_json::{Result, Value, Map};
use reqwest::blocking::{Client, ClientBuilder};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use reqwest::header::{HeaderMap, HeaderValue};
use std::vec::Vec;
use std::iter::FromIterator;

pub struct ApolloCloudClient {
    endpoint_url: String,
    auth_token: String,
    client: Client,
}

#[derive(Serialize)]
struct Query {
    query: String,
}

#[derive(Deserialize)]
struct GetOrgMembershipResponse__account {
    id: String
}

#[derive(Deserialize)]
struct GetOrgMembershipResponse__membership {
   account: GetOrgMembershipResponse__account
}

#[derive(Deserialize)]
struct GetOrgMembershipRespose__memberships {
  memberships: std::vec::Vec<GetOrgMembershipResponse__membership>
}

#[derive(Deserialize)]
struct GetOrgMembershipResponse__me {
   me: GetOrgMembershipRespose__memberships
}

#[derive(Deserialize)]
struct GetOrgMembershipResponse {
    data: GetOrgMembershipResponse__me
}

impl ApolloCloudClient {
    pub fn new(endpoint_url: String, auth_token: String) -> ApolloCloudClient {
        let client = Client::new();
        ApolloCloudClient {
            endpoint_url,
            auth_token,
            client,
        }
    }

    pub fn get_org_memberships(&self) -> HashSet<String> {
        let mut operation_map = HashMap::new();
        operation_map.insert("query", GET_ORG_MEMBERSHIPS_QUERY);
        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY",
                       HeaderValue::from_str(self.auth_token[..].as_ref()).unwrap());
        let res = match self.client.post("https://engine-staging-graphql.apollographql.com/api/graphql")
            .headers(headers)
            .json::<HashMap<&str, &str>>(&operation_map).send() {
            Ok(res) => res,
            Err(e) => panic!(e)
        };
        let text = res.text().unwrap();
        let results = serde_json::from_str::<GetOrgMembershipResponse>(&text).unwrap();
        HashSet::from_iter(results.data.me.memberships.into_iter().map(|it| it.account.id).collect::<Vec<String>>())
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
