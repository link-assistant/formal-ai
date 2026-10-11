//! Host-owned source receipt and reversible candidate ports; no filesystem/process imports.
use serde_json::Value;

/// Implementations are trusted host boundaries, never caller message metadata.
pub trait SourceOperationHost {
    type Receipt;
    fn accepted_sources(
        &self,
        receipt: &Self::Receipt,
        request: &str,
        command: &str,
        workspace: &str,
    ) -> Result<Vec<Value>, &'static str>;
}
/// Mirrors acceptedOperationSourceBytes: absent hosts refuse instead of inventing evidence.
pub fn accepted_operation_source_bytes<Host: SourceOperationHost>(
    host: Option<&Host>,
    receipt: &Host::Receipt,
    request: &str,
    command: &str,
    workspace: &str,
) -> Result<Vec<Value>, &'static str> {
    host.ok_or("MissingSourceOperationHost")?
        .accepted_sources(receipt, request, command, workspace)
}

/// Candidate proof is bound to bytes before/after the actual operation and at consumption.
pub trait SourceCandidateOperationHost: SourceOperationHost {
    fn candidate_status(
        &self,
        receipt: &Self::Receipt,
        request: &str,
        command: &str,
        workspace: &str,
        destination: &str,
        identity: &str,
    ) -> Result<Option<i32>, &'static str>;
}
/// Final delivery requires owned IO completion in addition to an accepted process receipt.
pub trait SourceFinalizedCandidateOperationHost: SourceCandidateOperationHost {
    fn finalized_candidate_status(
        &self,
        receipt: &Self::Receipt,
        request: &str,
        command: &str,
        workspace: &str,
        destination: &str,
        identity: &str,
    ) -> Result<Option<i32>, &'static str>;
}
/// Mirrors acceptedOperationCandidateStatus; caller text cannot manufacture status.
pub fn accepted_operation_candidate_status<Host: SourceFinalizedCandidateOperationHost>(
    host: Option<&Host>,
    receipt: &Host::Receipt,
    request: &str,
    command: &str,
    workspace: &str,
    destination: &str,
    identity: &str,
) -> Result<Option<i32>, &'static str> {
    host.ok_or("MissingSourceOperationHost")?
        .finalized_candidate_status(receipt, request, command, workspace, destination, identity)
}

#[derive(PartialEq)]
pub enum CandidateFile {
    Absent,
    File(String),
    Other,
}
pub enum ProcessDisposition {
    Succeeded,
    Failed,
    Incomplete,
}
pub trait CandidateIo {
    type Receipt;
    fn owns_lease(&self) -> bool;
    fn inspect(&self, path: &str) -> CandidateFile;
    fn process_disposition(
        &self,
        receipt: &Self::Receipt,
    ) -> Result<ProcessDisposition, &'static str>;
    fn remove_unchanged_candidate(&mut self, path: &str, identity: &str) -> bool;
}
pub struct CandidateTransaction<'host, Io: CandidateIo> {
    io: &'host mut Io,
    path: String,
    identity: String,
    phase: &'static str,
}
/// Mirrors createCandidateTransaction; the IO provider owns both lease and receipts.
pub fn create_candidate_transaction<Io: CandidateIo>(
    io: &mut Io,
    path: String,
    identity: String,
) -> Result<CandidateTransaction<'_, Io>, &'static str> {
    if !io.owns_lease() {
        return Err("MissingSingleWriterLease");
    }
    if io.inspect(&path) != CandidateFile::Absent {
        return Err("UnownedExistingDestination");
    }
    Ok(CandidateTransaction {
        io,
        path,
        identity,
        phase: "prepared",
    })
}
impl<Io: CandidateIo> CandidateTransaction<'_, Io> {
    fn unchanged(&self) -> bool {
        self.io.inspect(&self.path) == CandidateFile::File(self.identity.clone())
    }
    pub fn authorize_write(&self) -> Result<Value, &'static str> {
        if self.phase != "prepared" {
            return Err("InvalidCandidatePhase");
        }
        if !self.io.owns_lease() {
            return Err("MissingSingleWriterLease");
        }
        if self.io.inspect(&self.path) != CandidateFile::Absent {
            return Err("UnownedExistingDestination");
        }
        Ok(serde_json::json!({"state":self.phase,"authorized":true}))
    }
    pub fn record_write(&mut self) -> Result<Value, &'static str> {
        if !self.io.owns_lease() {
            return Err("MissingSingleWriterLease");
        }
        if self.phase != "prepared" {
            return Err("InvalidCandidatePhase");
        }
        if !self.unchanged() {
            return Err("UnobservedCandidateWrite");
        }
        self.phase = "written";
        Ok(serde_json::json!({"state":self.phase,"removed":false}))
    }
    /// Remove only the unchanged owned candidate without claiming any process ran.
    pub fn abort(&mut self) -> Result<Value, &'static str> {
        if self.phase != "written" {
            return Err("InvalidCandidatePhase");
        }
        if !self.io.owns_lease() {
            return Err("MissingSingleWriterLease");
        }
        if !self.unchanged() {
            self.phase = "refused-drift";
            return Ok(serde_json::json!({"state":self.phase,"removed":false}));
        }
        let removed = self
            .io
            .remove_unchanged_candidate(&self.path, &self.identity);
        self.phase = if removed {
            "rolled-back"
        } else {
            "refused-drift"
        };
        Ok(
            serde_json::json!({"state":self.phase,"removed":removed,"processDisposition":"unverified"}),
        )
    }
    pub fn finish(&mut self, receipt: &Io::Receipt) -> Result<Value, &'static str> {
        if self.phase != "written" {
            return Err("InvalidCandidatePhase");
        }
        if !self.io.owns_lease() {
            return Err("MissingSingleWriterLease");
        }
        let disposition = self.io.process_disposition(receipt)?;
        if !self.unchanged() {
            self.phase = "refused-drift";
            return Ok(serde_json::json!({"state":self.phase,"removed":false}));
        }
        if matches!(disposition, ProcessDisposition::Succeeded) {
            self.phase = "committed";
            return Ok(serde_json::json!({"state":self.phase,"removed":false}));
        }
        let removed = self
            .io
            .remove_unchanged_candidate(&self.path, &self.identity);
        self.phase = if removed {
            "rolled-back"
        } else {
            "refused-drift"
        };
        Ok(serde_json::json!({"state":self.phase,"removed":removed}))
    }
}

#[cfg(test)]
mod finalized_operation_tests {
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
}
