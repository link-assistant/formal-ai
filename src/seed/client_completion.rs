//! Data-defined completion contracts for external coding clients.

use super::embedded::CLIENT_COMPLETION_CONTRACTS_LINO;
use super::parser::parse_lino;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientCompletionContract {
    pub observable_postcondition: String,
    pub max_attempts: usize,
    pub incomplete_reason: String,
    pub scratch_directory: String,
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn software_authoring_contract_is_complete_and_bounded() {
        let contract = software_authoring_completion_contract().expect("completion contract");
        assert_eq!(contract.observable_postcondition, "workspace_effect");
        assert_eq!(contract.max_attempts, 2);
        assert_eq!(
            contract.incomplete_reason,
            "required_workspace_effect_missing"
        );
        assert_eq!(contract.scratch_directory, ".formal-ai");
    }
}
