use serde_json::{Value, Map, Error};
use reqwest::blocking::{Client, ClientBuilder};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use reqwest::header::{HeaderMap, HeaderValue};
use std::vec::Vec;
use std::iter::FromIterator;
use serde::de::DeserializeOwned;
use dirs::data_dir;

pub struct ApolloCloudClient {
    endpoint_url: String,
    client: Client,
}

#[derive(Serialize)]
struct CreateGraphVariables {
    graphID: String,
    accountID: String,
}

#[derive(Deserialize)]
struct CreateGraphResponseApiKey {
    token: String,
}

#[derive(Deserialize)]
struct CreateGraphResponseNewService {
    id: String,
    apiKeys: Vec<CreateGraphResponseApiKey>,
}

#[derive(Deserialize)]
struct CreateGraphResponseData {
    newService: CreateGraphResponseNewService,
}

#[derive(Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Deserialize)]
struct CreateGraphResponse {
    data: Option<CreateGraphResponseData>,
    errors: Option<Vec<GraphqlError>>,
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
    data: Option<GetOrgMembershipResponseMe>,
    errors: Option<Vec<GraphqlError>>,
}

pub struct GraphqlOperationError {
    message: String,
    user_error: bool,
}

impl ApolloCloudClient {
    pub fn new(endpoint_url: String, auth_token: String) -> ApolloCloudClient {
        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY",
                       HeaderValue::from_str(&auth_token[..].as_ref()).unwrap());

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build().unwrap();

        ApolloCloudClient {
            endpoint_url,
            client,
        }
    }

    fn execute_operation<T: DeserializeOwned, V: Serialize>(&self, operation_string: &str, variables: V) -> Result<T, Error> {
        let mut json_payload: HashMap<&str, String> = HashMap::new();
        json_payload.insert("query", String::from(operation_string));
        let vars_string = serde_json::to_string(&variables).unwrap();
        json_payload.insert("variables", vars_string);

        let res = match self.client.post(&self.endpoint_url)
            .json::<HashMap<&str, String>>(&json_payload).send() {
            Ok(res) => res,
            Err(e) => panic!(e)
        };

        let text = String::from(res.text().unwrap());
        match serde_json::from_str::<T>(&text) {
            Ok(r) => Ok(r),
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn execute_operation_no_variables<T: DeserializeOwned>(&self, operation_string: &str) -> Result<T, Error> {
        let mut json_payload: HashMap<&str, String> = HashMap::new();
        json_payload.insert("query", String::from(operation_string));

        let res = match self.client.post(&self.endpoint_url)
            .json::<HashMap<&str, String>>(&json_payload).send() {
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
        let result = match self.execute_operation_no_variables::<GetOrgMembershipResponse>(
            GET_ORG_MEMBERSHIPS_QUERY) {
            Ok(r) => r,
            Err(e) => {
                println!("Encountered error {}", e);
                return Err("Could not fetch organizations");
            }
        };
        match result.data.unwrap().me {
            Some(me) =>
                Ok(
                    HashSet::from_iter(
                        me.memberships.into_iter().map(
                            |it| it.account.id
                        ).collect::<Vec<String>>())),
            None => Err("Could not authenticate. Please check that your auth token is up-to-date"),
        }
    }

    pub fn create_new_graph(&self, graph_id: String, account_id: String) -> Result<String, GraphqlOperationError> {
        let variables = CreateGraphVariables {
            graphID: graph_id,
            accountID: account_id,
        };
        let result =
            match self.execute_operation::<CreateGraphResponse, CreateGraphVariables>(CREATE_GRAPH_QUERY, variables) {
                Ok(result) => result,
                Err(message) => return Err(GraphqlOperationError { message: message.to_string(), user_error: false })
            };
        if result.errors.is_some() {
            let message = result.errors.unwrap()
                .iter_mut().map(| err| err.message.clone())
                .collect::<Vec<String>>().join("\n");
            return Err(GraphqlOperationError { message, user_error: false });
        }

        let data = match result.data {
            Some(data) => data,
            None => return Err(GraphqlOperationError {
                message: String::from("Got no data????"),
                user_error: false,
            })
        };

        Ok(data.newService.apiKeys[0].token.clone())
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
