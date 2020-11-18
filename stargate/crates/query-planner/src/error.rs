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
    #[error("expected a field where there was none")]
    MissingField,
    #[error("expected operation in document where there was none")]
    MissingOperation,
    #[error("expected a return type where there was none")]
    MissingReturnType,
    #[error("expected a group where there was none")]
    MissingGroup,
    #[error("expected a type name for a field where there was none")]
    MissingTypeNameForField,
    #[error("TODO")]
    TODO,
}
