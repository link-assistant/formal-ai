//! Native source consumer over maintained Read history and trusted host/IO ports.
use super::complete_source_preflight::{
    bind_complete_source_needs, derive_complete_source_request,
};
use super::seeded_source_plan::derive_seeded_source_plan;
use super::source_module_identity_port::SourceModuleIdentityHost;
use super::source_need_preflight::source_need_preflight;
use super::source_operation_transaction::{
    CandidateIo, CandidateTransaction, SourceOperationHost, create_candidate_transaction,
};
use crate::agentic_coding::{
    planner::{self, AgenticPlan},
    progress::Progress,
};
use crate::protocol::ChatMessage;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

/// Only the installed host supplies operation metadata and authored grammar.
pub trait SourceCompositionHost: SourceOperationHost + SourceModuleIdentityHost {
    fn request_seed(&self) -> &str;
    fn composition_seed(&self) -> &str;
    fn operation_descriptor(
        &self,
        receipt: &Self::Receipt,
        request: &str,
        command: &str,
        workspace: &str,
    ) -> Result<Value, &'static str>;
}
pub struct PreparedSourceCandidate<'io, Io: CandidateIo> {
    source_identity: String,
    request_text: String,
    command: String,
    workspace: String,
    destination: String,
    pub plan: AgenticPlan,
    pub composition: Value,
    pub conditional_proof: Value,
    pub transaction: CandidateTransaction<'io, Io>,
}
/// Missing hosts, incomplete Reads, unknown Needs and unowned destinations refuse before Write.
pub fn prepare_source_candidate<'io, Host: SourceCompositionHost, Io: CandidateIo>(
    host: Option<&Host>,
    io: &'io mut Io,
    receipt: &Host::Receipt,
    request_text: &str,
    messages: &[ChatMessage],
    workspace: &str,
    tool_names: &[&str],
) -> Result<PreparedSourceCandidate<'io, Io>, &'static str> {
    let host = host.ok_or("MissingSourceOperationHost")?;
    let write_tool =
        crate::agentic_coding::capability_router::tool_for(tool_names, planner::Capability::Write)
            .ok_or("MissingWriteTool")?;
    let frame = derive_complete_source_request(request_text, host.request_seed())
        .ok_or("UnboundCompleteSourceRequest")?;
    let preflight =
        source_need_preflight(request_text, messages).ok_or("UnboundSourceReadHistory")?;
    if !preflight["prerequisites"].as_array().is_some_and(|values| {
        values.iter().all(|value| {
            matches!(
                value["status"].as_str(),
                Some("conditional-satisfied-read" | "conditional-satisfied-absence")
            )
        })
    }) {
        return Err("UnsolvedImmutableReadPrerequisite");
    }
    if !preflight["prerequisites"].as_array().is_some_and(|values| {
        values.iter().any(|value| {
            value["role"] == "destination" && value["status"] == "conditional-satisfied-absence"
        })
    }) {
        return Err("UnprovedDestinationMerge");
    }
    let request = &frame["request"];
    let command = request["command"]
        .as_str()
        .ok_or("MissingAcceptedCommand")?;
    let canonical_sources = host.accepted_sources(receipt, request_text, command, workspace)?;
    let operation = host.operation_descriptor(receipt, request_text, command, workspace)?;
    let progress = Progress::scan(messages);
    let inputs = request["inputs"].as_array().ok_or("MissingSourceInputs")?;
    let mut observations = Vec::new();
    for input in inputs {
        let path = input.as_str().ok_or("MissingSourcePath")?;
        let read = progress.source_read_for(path).ok_or("MissingSourceRead")?;
        let bytes = read
            .source
            .as_ref()
            .filter(|_| read.complete && read.error.is_none())
            .ok_or("IncompleteSourceRead")?;
        let identity = sha256_hex(bytes.as_bytes());
        let matching: Vec<_> = canonical_sources
            .iter()
            .filter(|source| source["sha256"] == identity && source["content"] == *bytes)
            .collect();
        if matching.len() != 1 {
            return Err("AmbiguousCanonicalSourceCorrespondence");
        }
        observations.push(json!({"path":matching[0]["path"],"content":bytes}));
    }
    let acceptance_paths = request["acceptance"]
        .as_array()
        .filter(|paths| paths.len() == 1)
        .ok_or("AmbiguousAcceptance")?;
    let acceptance_path = acceptance_paths[0]
        .as_str()
        .ok_or("MissingAcceptancePath")?;
    let acceptance_read = progress
        .source_read_for(acceptance_path)
        .ok_or("MissingAcceptanceRead")?;
    let acceptance_bytes = acceptance_read
        .source
        .as_ref()
        .filter(|_| acceptance_read.complete && acceptance_read.error.is_none())
        .ok_or("IncompleteAcceptanceRead")?;
    let acceptance = json!({"path":acceptance_path,"content":acceptance_bytes});
    let mut qualified_request = request.clone();
    let fields = qualified_request
        .as_object_mut()
        .ok_or("MissingRequestObject")?;
    fields.insert("text".to_owned(), json!(request_text));
    fields.insert("identity".to_owned(), frame["requestIdentity"].clone());
    let composition = derive_seeded_source_plan(
        host,
        &qualified_request,
        &observations,
        &acceptance,
        &operation,
        host.composition_seed(),
    )?;
    let conditional_proof =
        bind_complete_source_needs(&frame, &composition, &preflight, &canonical_sources)?;
    // Revalidate the same private receipt after every source-derived contract calculation.
    if host.accepted_sources(receipt, request_text, command, workspace)? != canonical_sources {
        return Err("CanonicalSourceDrift");
    }
    let destination = request["destination"]
        .as_str()
        .ok_or("MissingDestination")?;
    let source = composition["source"]
        .as_str()
        .ok_or("MissingComposedSource")?;
    let identity = sha256_hex(source.as_bytes());
    let transaction = create_candidate_transaction(io, destination.to_owned(), identity.clone())?;
    let plan = planner::plan_one(
        write_tool,
        json!({"path":destination,"content":source}).to_string(),
    );
    Ok(PreparedSourceCandidate {
        source_identity: identity,
        request_text: request_text.to_owned(),
        command: command.to_owned(),
        workspace: workspace.to_owned(),
        destination: destination.to_owned(),
        plan,
        composition,
        conditional_proof,
        transaction,
    })
}

/// Consume only trusted candidate-bound status; public success prose remains unverified.
pub fn finish_source_candidate<Host, Io>(
    host: Option<&Host>,
    prepared: &mut PreparedSourceCandidate<'_, Io>,
    receipt: &Host::Receipt,
    process_receipt: &Io::Receipt,
) -> Result<Value, &'static str>
where
    Host: super::source_operation_transaction::SourceCandidateOperationHost,
    Io: CandidateIo,
{
    let identity = &prepared.source_identity;
    let status = host.ok_or("MissingSourceOperationHost").and_then(|host| {
        host.candidate_status(
            receipt,
            &prepared.request_text,
            &prepared.command,
            &prepared.workspace,
            &prepared.destination,
            identity,
        )
    });
    match status {
        Ok(Some(0)) => {
            let disposition = match prepared.transaction.finish(process_receipt) {
                Ok(disposition) => disposition,
                Err(error) => {
                    let rollback = prepared.transaction.abort()?;
                    return Ok(json!({"authored":true,"verified":false,"ownedStatus":0,
                        "verificationFailure":error,"physicalDisposition":rollback,
                        "universalEffectsProved":false}));
                }
            };
            let verified = disposition["state"] == "committed";
            Ok(json!({"authored":true,"verified":verified,"ownedStatus":0,
                "physicalDisposition":disposition,"universalEffectsProved":false}))
        }
        observation => {
            let disposition = prepared.transaction.abort()?;
            Ok(json!({"authored":true,"verified":false,
                "ownedStatus":observation.as_ref().ok().copied().flatten(),
                "verificationFailure":observation.err(),"physicalDisposition":disposition,
                "universalEffectsProved":false}))
        }
    }
}

/// Mirrors recordPreparedSourceWrite; no caller metadata can create this transaction.
pub fn authorize_source_candidate_write<Io: CandidateIo>(
    prepared: &PreparedSourceCandidate<'_, Io>,
) -> Result<Value, &'static str> {
    prepared.transaction.authorize_write()
}
pub fn record_prepared_source_write<Io: CandidateIo>(
    prepared: &mut PreparedSourceCandidate<'_, Io>,
) -> Result<Value, &'static str> {
    prepared.transaction.record_write()
}

/// Mirrors abortSourceCandidate: receipt absence is unverified, not successful execution.
pub fn abort_source_candidate<Io: CandidateIo>(
    prepared: &mut PreparedSourceCandidate<'_, Io>,
) -> Result<Value, &'static str> {
    let disposition = prepared.transaction.abort()?;
    Ok(json!({"authored":true,"verified":false,"ownedStatus":null,
       "verificationFailure":"MissingOwnedVerificationReceipt","physicalDisposition":disposition,
       "universalEffectsProved":false}))
}

/// Trusted execution callbacks consume the same private prepared transaction as the JavaScript session.
pub struct SourceCandidateContext<'context> {
    pub request: &'context str,
    pub messages: &'context [ChatMessage],
    pub workspace: &'context str,
    pub tools: &'context [&'context str],
}
pub fn execute_source_candidate<Host, Io, Write, Verify>(
    host: Option<&Host>,
    io: &mut Io,
    receipt: &Host::Receipt,
    context: SourceCandidateContext<'_>,
    mut execute_write: Write,
    mut execute_verification: Verify,
) -> Result<Value, &'static str>
where
    Host: SourceCompositionHost + super::source_operation_transaction::SourceCandidateOperationHost,
    Io: CandidateIo,
    Write: FnMut(&AgenticPlan) -> Result<(), &'static str>,
    Verify: FnMut(&str, &str) -> Result<(Host::Receipt, Io::Receipt), &'static str>,
{
    let SourceCandidateContext {
        request,
        messages,
        workspace,
        tools,
    } = context;
    let mut prepared =
        prepare_source_candidate(host, io, receipt, request, messages, workspace, tools)?;
    authorize_source_candidate_write(&prepared)?;
    if let Err(error) = execute_write(&prepared.plan) {
        record_prepared_source_write(&mut prepared).map_err(|_| "UnownedWriteFailure")?;
        let mut outcome = abort_source_candidate(&mut prepared)?;
        outcome["executionFailure"] = json!(error);
        return Ok(outcome);
    }
    record_prepared_source_write(&mut prepared)?;
    let verification = execute_verification(&prepared.command, &prepared.workspace);
    match verification {
        Ok((after, process)) => finish_source_candidate(host, &mut prepared, &after, &process),
        Err(error) => {
            let mut outcome = abort_source_candidate(&mut prepared)?;
            outcome["executionFailure"] = json!(error);
            Ok(outcome)
        }
    }
}
