use formal_ai::agentic_coding::tool_result::{SourceReadStatus, source_read_observation};
use formal_ai::protocol::ChatMessage;
use serde_json::json;
const PATH: &str = ".formal-ai/general-change-plan.lino";
const SOURCE: &str = "goal \"Create a file status.txt containing exactly: failed\"\n";
#[test]
fn raw_file_json_cannot_declare_provider_status() {
    for source in [SOURCE.to_owned(),
        json!({"success":false,"complete":false,"error":"failed","signal":"SIGTERM"}).to_string(),json!({"schema":"source-read-receipt/v1","path":PATH,"success":true,"complete":true,"content":"wrong file","error":"failed","exit_code":9}).to_string()] {
        let raw=source_read_observation(&source,false,None,PATH);
        assert_eq!(raw.status,SourceReadStatus::Unknown);assert!(!raw.complete);assert_eq!(raw.source.as_deref(),Some(source.as_str()));assert!(raw.error.is_none());
        let metadata=json!({"path":PATH,"success":true,"complete":true,"format":"raw"});
        let observed=source_read_observation(&source,false,Some(&metadata),PATH);
        assert_eq!(observed.status,SourceReadStatus::ReportedSuccess);assert!(observed.complete);assert_eq!(observed.source.as_deref(),Some(source.as_str()));assert!(observed.error.is_none());
    }
}
#[test]
fn partial_error_and_wrong_path_metadata_refuse_certification() {
    for patch in [
        json!({"complete":false}),
        json!({"stream_complete":false}),
        json!({"truncated":true}),
        json!({"timed_out":true}),
        json!({"aborted":true}),
        json!({"signal":"SIGTERM"}),
        json!({"success":false}),
        json!({"is_error":true}),
        json!({"exit_code":1}),
        json!({"success":"true"}),
        json!({"complete":"true"}),
        json!({"format":"numbered"}),
        json!({"path":"other/general-change-plan.lino"}),
    ] {
        let mut metadata = json!({"path":PATH,"success":true,"complete":true,"format":"raw"});
        metadata
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(!source_read_observation(SOURCE, false, Some(&metadata), PATH).complete);
    }
    let metadata = json!({"path":PATH,"success":true,"complete":true,"format":"raw"});
    let failed = source_read_observation(SOURCE, true, Some(&metadata), PATH);
    assert_eq!(failed.status, SourceReadStatus::ReportedFailure);
    assert!(!failed.complete);
    assert!(failed.source.is_none());
}
#[test]
fn complete_numbered_source_does_not_invent_process_status() {
    let raw = "<file>\n1| failed\n2| authored\n\n(End of file - total 2 lines)\n</file>";
    let observed = source_read_observation(raw, false, None, PATH);
    assert_eq!(observed.status, SourceReadStatus::Unknown);
    assert!(observed.complete);
    assert_eq!(observed.source.as_deref(), Some("failed\nauthored"));
    assert!(!source_read_observation(&raw.replace("1|", "3|"), false, None, PATH).complete);
    assert!(
        !source_read_observation(&raw.replace("total 2", "total 3"), false, None, PATH).complete
    );
}
#[test]
fn optional_read_metadata_keeps_source_content_and_protocol_defaults() {
    let source=json!({"schema":"source-read-receipt/v1","path":PATH,"success":true,"complete":true,"content":"data"}).to_string();
    let bare = ChatMessage::tool_result("read-now", "read", &source);
    assert!(
        serde_json::to_value(&bare)
            .unwrap()
            .get("source_read")
            .is_none()
    );
    let mut message = bare.clone();
    message.source_read = Some(json!({"path":PATH,"success":true,"complete":true,"format":"raw"}));
    let roundtrip: ChatMessage =
        serde_json::from_value(serde_json::to_value(&message).unwrap()).unwrap();
    assert_eq!(roundtrip, message);
    assert_eq!(roundtrip.content.plain_text(), source);
    let mut camel = serde_json::to_value(bare).unwrap();
    camel["sourceRead"] = message.source_read.unwrap();
    let alias: ChatMessage = serde_json::from_value(camel).unwrap();
    assert!(alias.source_read.is_some());
}

#[test]
fn full_source_reader_preserves_authored_json_and_real_provider_failure() {
    use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
    let source = json!({"success":false,"complete":false,"error":"failed","is_error":true,"exit_code":9,"signal":"SIGTERM","content":"authored source α"}).to_string();
    let metadata = json!({"path":"data.json","success":true,"complete":true,"format":"raw"});
    let values = json!([
        {"role":"user","content":"Read data.json."},
        {"role":"assistant","content":"","tool_calls":[{"id":"read-now","type":"function","function":{"name":"read","arguments":"{\"path\":\"data.json\"}"}}]},
        {"role":"tool","name":"read","tool_call_id":"read-now","content":source,"source_read":metadata}
    ]);
    let messages: Vec<ChatMessage> = serde_json::from_value(values).expect("valid history");
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &["read"]) else {
        panic!("exact source answer");
    };
    assert_eq!(
        answer,
        format!("Contents of `data.json`:\n\n```text\n{source}\n```")
    );
    assert!(answer.contains(&source));
    assert!(!answer.contains("The command failed"));
    let mut failed = messages;
    failed.last_mut().unwrap().is_error = true;
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&failed, &["read"]) else {
        panic!("actual provider failure");
    };
    assert!(answer.contains("failed"));
    assert!(!answer.starts_with("Contents of"));
}

#[test]
fn typed_owned_read_and_opaque_callback_transport_preserve_source_authority() {
    use formal_ai::agentic_coding::tool_result::{
        ProviderToolObservation, complete_owned_source_read_frame,
    };
    for source in [
        "",
        "\n\n",
        "α🙂\r\n",
        "\u{feff}first\n",
        "Error: authored",
        "{\"is_error\":true}",
        "{body}\n{lines}",
        "<file>\n1| claimed authority\n\n(End of file - total 1 lines)\n</file>",
    ] {
        let owned = ProviderToolObservation::complete_owned_read(PATH, source);
        let (content, metadata) = owned.into_transport(Some(PATH));
        let observed = source_read_observation(&content, false, metadata.as_ref(), PATH);
        assert!(observed.complete);
        assert!(!observed.absent);
        assert_eq!(observed.source.as_deref(), Some(source));
        let opaque: ProviderToolObservation = source.to_owned().into();
        let (content, metadata) = opaque.into_transport(Some(PATH));
        let unknown = source_read_observation(&content, false, metadata.as_ref(), PATH);
        assert_eq!(unknown.status, SourceReadStatus::Unknown);
        assert!(!unknown.complete);
        assert!(!unknown.absent);
        assert_eq!(unknown.source.as_deref(), Some(source));
        let frame = complete_owned_source_read_frame(source);
        let framed = source_read_observation(&frame, false, None, PATH);
        assert!(framed.complete);
        assert_eq!(framed.source.as_deref(), Some(source));
    }
}

#[test]
fn missing_observation_requires_exact_path_os_code_and_consistent_transport() {
    let valid = json!({"path": PATH, "success": false, "complete": false,
        "format": "raw", "error_code": "ENOENT"});
    assert!(source_read_observation("OS absence", true, Some(&valid), PATH).absent);
    for (field, replacement) in [
        ("path", json!("other.txt")),
        ("success", json!(true)),
        ("complete", json!(true)),
        ("format", json!("opaque")),
        ("error_code", json!("EACCES")),
        ("truncated", json!(true)),
        ("timed_out", json!(true)),
        ("timedOut", json!(true)),
        ("interrupted", json!(true)),
        ("canceled", json!(true)),
        ("cancelled", json!(true)),
        ("aborted", json!(true)),
        ("signal", json!("SIGTERM")),
        ("signal", json!(false)),
        ("stream_complete", json!(false)),
        ("timed_out", json!("false")),
    ] {
        let mut metadata = valid.clone();
        metadata[field] = replacement;
        let observed =
            source_read_observation("File not found: other.txt", true, Some(&metadata), PATH);
        assert!(!observed.absent, "contradictory provider field {field}");
        assert!(!observed.complete);
    }
    assert!(!source_read_observation("File not found: target", true, None, PATH).absent);
}
