//! Data-defined completion contracts for external coding clients.

use super::embedded::CLIENT_COMPLETION_CONTRACTS_LINO;
use super::parser::parse_lino;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientCompletionContract {
    pub observable_postcondition: String,
    pub max_attempts: usize,
    pub incomplete_reason: String,
    pub scratch_directory: String,
    pub incomplete_error: String,
    pub process_error: String,
}

#[must_use]
pub fn software_authoring_completion_contract() -> Option<ClientCompletionContract> {
    let tree = parse_lino(CLIENT_COMPLETION_CONTRACTS_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "completion_contracts")?;
    let contract = root
        .children
        .iter()
        .find(|node| node.name == "software_authoring")?;
    Some(ClientCompletionContract {
        observable_postcondition: contract
            .find_child_value("observable_postcondition")
            .to_owned(),
        max_attempts: contract.find_child_value("max_attempts").parse().ok()?,
        incomplete_reason: contract.find_child_value("incomplete_reason").to_owned(),
        scratch_directory: contract.find_child_value("scratch_directory").to_owned(),
        incomplete_error: contract.find_child_value("incomplete_error").to_owned(),
        process_error: contract.find_child_value("process_error").to_owned(),
    })
}
