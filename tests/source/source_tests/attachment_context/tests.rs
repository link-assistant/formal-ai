use super::*;

#[test]
fn human_readable_size_scales_units() {
    assert_eq!(human_readable_size(512), "512 B");
    assert_eq!(human_readable_size(8 * 1024), "8.0 KB");
    assert_eq!(human_readable_size(3 * 1024 * 1024 + 512 * 1024), "3.5 MB");
}

#[test]
fn builds_block_with_size_and_excerpt() {
    let attachments = [Attachment::new("report.txt", "text/plain")
        .with_size(8 * 1024)
        .with_excerpt("The tower opened in 1889.")];
    let block = build_attachment_context(&attachments).expect("block");
    assert_eq!(
        block,
        "Attached files:\n1. report.txt (text/plain, 8.0 KB)\n\
         Text excerpt: The tower opened in 1889.",
    );
}

#[test]
fn builds_block_without_size_falls_back_to_mime_only() {
    let attachments = [Attachment::new("photo.jpg", "image/jpeg")];
    let block = build_attachment_context(&attachments).expect("block");
    assert_eq!(block, "Attached files:\n1. photo.jpg (image/jpeg)");
}

#[test]
fn empty_mime_falls_back_to_octet_stream() {
    let attachments = [Attachment::new("blob", "")];
    let block = build_attachment_context(&attachments).expect("block");
    assert_eq!(block, "Attached files:\n1. blob (application/octet-stream)");
}

#[test]
fn numbers_multiple_files() {
    let attachments = [
        Attachment::new("a.txt", "text/plain").with_size(1024),
        Attachment::new("b.txt", "text/plain").with_size(2048),
    ];
    let block = build_attachment_context(&attachments).expect("block");
    assert!(block.contains("\n1. a.txt (text/plain, 1.0 KB)"));
    assert!(block.contains("\n2. b.txt (text/plain, 2.0 KB)"));
}

#[test]
fn no_attachments_yields_none() {
    assert!(build_attachment_context(&[]).is_none());
}

#[test]
fn composes_message_then_context() {
    let attachments = [Attachment::new("report.txt", "text/plain").with_size(8 * 1024)];
    let prompt = compose_prompt_with_attachments(Some("Check this"), &attachments).expect("prompt");
    assert_eq!(
        prompt,
        "Check this\n\nAttached files:\n1. report.txt (text/plain, 8.0 KB)",
    );
}

#[test]
fn composes_context_only_when_no_message() {
    let attachments = [Attachment::new("report.txt", "text/plain")];
    let prompt = compose_prompt_with_attachments(None, &attachments).expect("prompt");
    assert_eq!(prompt, "Attached files:\n1. report.txt (text/plain)");
}

#[test]
fn composes_message_only_when_no_attachment() {
    assert_eq!(
        compose_prompt_with_attachments(Some("just text"), &[]),
        Some(String::from("just text")),
    );
}

#[test]
fn nothing_to_compose_yields_none() {
    assert_eq!(compose_prompt_with_attachments(None, &[]), None);
    assert_eq!(compose_prompt_with_attachments(Some("   "), &[]), None);
}
