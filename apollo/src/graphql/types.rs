use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct CreateGraphVariables {
    pub graphID: String,
    pub accountID: String,
}

#[derive(Deserialize)]
pub struct CreateGraphResponseApiKey {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateGraphResponseNewService {
    pub id: String,
    pub apiKeys: Vec<CreateGraphResponseApiKey>,
}

#[derive(Deserialize)]
pub struct CreateGraphResponseData {
    pub newService: CreateGraphResponseNewService,
}

#[derive(Deserialize)]
pub struct GraphqlError {
    pub message: String,
}

#[derive(Deserialize)]
pub struct CreateGraphResponse {
    pub data: Option<CreateGraphResponseData>,
    pub errors: Option<Vec<GraphqlError>>,
}

#[derive(Deserialize)]
pub struct GetOrgMembershipResponseAccount {
    pub id: String
}

#[derive(Deserialize)]
pub struct GetOrgMembershipResponseMembership {
    pub account: GetOrgMembershipResponseAccount
}

#[derive(Deserialize)]
pub struct GetOrgMembershipResposeMemberships {
    pub memberships: std::vec::Vec<GetOrgMembershipResponseMembership>
}

#[derive(Deserialize)]
pub struct GetOrgMembershipResponseMe {
    pub me: Option<GetOrgMembershipResposeMemberships>
}

#[derive(Deserialize)]
pub struct GetOrgMembershipResponse {
    pub data: Option<GetOrgMembershipResponseMe>,
    pub errors: Option<Vec<GraphqlError>>,
}

