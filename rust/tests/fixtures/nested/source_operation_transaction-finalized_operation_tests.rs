#![cfg(test)]

use super::*;
use std::rc::Rc;

struct Receipt(Rc<()>);
struct ProofHost {
    owned: Rc<()>,
    finalized: bool,
}
impl SourceOperationHost for ProofHost {
    type Receipt = Receipt;
    fn accepted_sources(
        &self,
        receipt: &Receipt,
        _request: &str,
        _command: &str,
        _workspace: &str,
    ) -> Result<Vec<Value>, &'static str> {
        if !Rc::ptr_eq(&self.owned, &receipt.0) {
            return Err("UnownedProcessReceipt");
        }
        Ok(Vec::new())
    }
}
impl SourceCandidateOperationHost for ProofHost {
    fn candidate_status(
        &self,
        receipt: &Receipt,
        request: &str,
        command: &str,
        workspace: &str,
        _destination: &str,
        _identity: &str,
    ) -> Result<Option<i32>, &'static str> {
        self.accepted_sources(receipt, request, command, workspace)?;
        Ok(Some(0))
    }
}
impl SourceFinalizedCandidateOperationHost for ProofHost {
    fn finalized_candidate_status(
        &self,
        receipt: &Receipt,
        request: &str,
        command: &str,
        workspace: &str,
        destination: &str,
        identity: &str,
    ) -> Result<Option<i32>, &'static str> {
        let status =
            self.candidate_status(receipt, request, command, workspace, destination, identity)?;
        Ok(if self.finalized { status } else { None })
    }
}
#[test]
fn successful_process_requires_finalized_ownership() {
    let owned = Rc::new(());
    let receipt = Receipt(Rc::clone(&owned));
    let mut host = ProofHost {
        owned,
        finalized: false,
    };
    assert_eq!(
        host.candidate_status(
            &receipt,
            "request",
            "command",
            "workspace",
            "destination",
            "identity"
        ),
        Ok(Some(0))
    );
    assert_eq!(
        accepted_operation_candidate_status(
            Some(&host),
            &receipt,
            "request",
            "command",
            "workspace",
            "destination",
            "identity"
        ),
        Ok(None)
    );
    host.finalized = true;
    assert_eq!(
        accepted_operation_candidate_status(
            Some(&host),
            &receipt,
            "request",
            "command",
            "workspace",
            "destination",
            "identity"
        ),
        Ok(Some(0))
    );
}
#[test]
fn absent_host_and_foreign_receipt_refuse() {
    let host = ProofHost {
        owned: Rc::new(()),
        finalized: true,
    };
    let foreign = Receipt(Rc::new(()));
    assert_eq!(
        accepted_operation_candidate_status::<ProofHost>(
            None,
            &foreign,
            "request",
            "command",
            "workspace",
            "destination",
            "identity"
        ),
        Err("MissingSourceOperationHost")
    );
    assert_eq!(
        accepted_operation_candidate_status(
            Some(&host),
            &foreign,
            "request",
            "command",
            "workspace",
            "destination",
            "identity"
        ),
        Err("UnownedProcessReceipt")
    );
}
