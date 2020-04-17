use serde_json::{Value, Map, Error};
use reqwest::blocking::{Client, ClientBuilder};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use reqwest::header::{HeaderMap, HeaderValue};
use std::vec::Vec;
use std::iter::FromIterator;
use serde::de::DeserializeOwned;

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
struct GetOrgMembershipResponseAccount {
    id: String
}

#[derive(Deserialize)]
struct GetOrgMembershipResponseMembership {
   account: GetOrgMembershipResponseAccount
}

#[derive(Deserialize)]
struct GetOrgMembershipResposeMemberships {
  memberships: std::vec::Vec<GetOrgMembershipResponseMembership>
}

#[derive(Deserialize)]
struct GetOrgMembershipResponseMe {
   me: Option<GetOrgMembershipResposeMemberships>
}

#[derive(Deserialize)]
struct GetOrgMembershipResponse {
    data: GetOrgMembershipResponseMe
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

    fn execute_operation<T: DeserializeOwned>(&self, operation_string: &str, variables: Option<HashMap<String,String>>) -> Result<T, Error> {
        let mut json_payload = HashMap::new();
        json_payload.insert("query", operation_string);
        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY",
                       HeaderValue::from_str(&self.auth_token[..].as_ref()).unwrap());
        let res = match self.client.post(&self.endpoint_url)
            .headers(headers)
            .json::<HashMap<&str, &str>>(&json_payload).send() {
            Ok(res) => res,
            Err(e) => panic!(e)
        };
        let text = String::from(res.text().unwrap());
        match serde_json::from_str::<T>(&text) {
            Ok(r) => Ok(r),
            Err(e) => {
                panic!(format!("Invalid response from Apollo cloud!\n{}", e))
            }
        }
    }

    pub fn get_org_memberships(&self) -> Result<HashSet<String>, &str> {
        let result = match self.execute_operation::<GetOrgMembershipResponse>(
            GET_ORG_MEMBERSHIPS_QUERY, None) {
            Ok(r) => r,
            Err(e) => {
                println!("Encountered error {}", e);
                return Err("Could not fetch organizations")
            },
        };
        match result.data.me {
            Some(me) =>
                Ok(
                    HashSet::from_iter(
                        me.memberships.into_iter().map(
                            |it| it.account.id
                        ).collect::<Vec<String>>())),
            None => Err("Could not authenticate. Please check that your auth token is up-to-date"),
        }

    }

    pub fn create_new_graph(&self) -> Result<&str, &str> {
        panic!("Not implemented!");
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

static CREATE_GRAPH_QUERY: &'static str = "
mutation CreateGraph($accountID: ID!, $graphID: ID!) {
  newService(accountId: $accountID, id: $graphID) {
    id
    apiKeys {
      token
    }
  }
}
";