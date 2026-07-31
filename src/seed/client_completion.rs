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
    /// Ordered recovery strategies, cheapest and most literal first. A run that
    /// missed the postcondition retries under the *next* strategy, never the
    /// same one twice, so a retry is a different decomposition rather than a
    /// repetition (issue #879).
    pub recovery_strategies: Vec<String>,
    /// Public vendor endpoints a local-server invocation must never reach. The
    /// list is data so a seventh client or a new vendor closes the whole class
    /// by declaring a row here.
    pub diverted_endpoints: Vec<String>,
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
    let values = |name: &str| {
        contract
            .children
            .iter()
            .filter(|child| child.name == name)
            .map(|child| child.id.clone())
            .collect::<Vec<_>>()
    };
    Some(ClientCompletionContract {
        observable_postcondition: contract
            .find_child_value("observable_postcondition")
            .to_owned(),
        max_attempts: contract.find_child_value("max_attempts").parse().ok()?,
        incomplete_reason: contract.find_child_value("incomplete_reason").to_owned(),
        scratch_directory: contract.find_child_value("scratch_directory").to_owned(),
        incomplete_error: contract.find_child_value("incomplete_error").to_owned(),
        process_error: contract.find_child_value("process_error").to_owned(),
        recovery_strategies: values("recovery_strategy"),
        diverted_endpoints: values("diverted_endpoint"),
    })
}
