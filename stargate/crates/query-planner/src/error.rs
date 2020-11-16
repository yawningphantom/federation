use graphql_parser::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryPlanError {
    #[error("failed parsing schema")]
    FailedParsingSchema(ParseError),
    #[error("failed parsing query")]
    FailedParsingQuery(ParseError),
    #[error("invalid query, multiple operations aren't supported")]
    InvalidQueryMultipleOperations,
    #[error("invalid query, subscriptions aren't supported")]
    InvalidQuerySubscriptions,
}
