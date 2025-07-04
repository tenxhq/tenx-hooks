use tenx_hooks::transcript::{
    TranscriptEntry, TranscriptMessage, parse_transcript_line, parse_transcript_with_context,
};

#[test]
fn test_tool_result_with_array_content() {
    let json_line = r#"{"parentUuid":"9a6212a9-b1d1-4823-b783-0c5248c2602c","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx-hooks/sandbox","sessionId":"bea16101-83bb-4150-b58d-295de3267bf9","version":"1.0.43","type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01UjUUPPebuG5bkbrpDPFTfQ","type":"tool_result","content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}]}]},"uuid":"c5c77671-8174-4149-abcc-650cc6058924","timestamp":"2025-07-04T10:59:44.274Z","toolUseResult":{"content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}],"totalDurationMs":10185,"totalTokens":11766,"totalToolUseCount":2,"usage":{"input_tokens":7,"cache_creation_input_tokens":153,"cache_read_input_tokens":11543,"output_tokens":63,"service_tier":"standard"},"wasInterrupted":false}}"#;

    match parse_transcript_line(json_line) {
        Ok(entry) => {
            println!("Successfully parsed: {}", entry.description());
            // Verify it's a user entry
            match entry {
                TranscriptEntry::User(_) => {
                    // Test passes
                }
                _ => panic!("Expected user entry"),
            }
        }
        Err(e) => {
            panic!("Failed to parse transcript: {e}");
        }
    }
}

#[test]
fn test_parse_various_entry_types() {
    // Test system entry
    let system_json = r#"{"type":"system","uuid":"sys-uuid","timestamp":"2025-01-01T00:00:00Z","content":"test content","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","isMeta":false,"level":"info","toolUseID":"tool-123"}"#;
    let system_entry = parse_transcript_line(system_json).expect("Should parse system entry");
    assert!(matches!(system_entry, TranscriptEntry::System(_)));

    // Test user entry
    let user_json = r#"{"type":"user","message":{"role":"user","content":"hello"},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":null}"#;
    let user_entry = parse_transcript_line(user_json).expect("Should parse user entry");
    assert!(matches!(user_entry, TranscriptEntry::User(_)));

    // Test assistant entry
    let assistant_json = r#"{"type":"assistant","message":{"id":"msg_123","type":"message","role":"assistant","model":"test-model","content":"hello back","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":2,"service_tier":"standard"}},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","requestId":"req-123"}"#;
    let assistant_entry =
        parse_transcript_line(assistant_json).expect("Should parse assistant entry");
    assert!(matches!(assistant_entry, TranscriptEntry::Assistant(_)));

    // Test summary entry - Summary doesn't have CommonFields, just summary and optional leafUuid
    let summary_json = r#"{"type":"summary","summary":"Test summary","leafUuid":"leaf-123"}"#;
    let summary_entry = parse_transcript_line(summary_json).expect("Should parse summary entry");
    assert!(matches!(summary_entry, TranscriptEntry::Summary(_)));
}

#[test]
fn test_parse_transcript_with_context_errors() {
    // Test with mixed valid and invalid lines
    let transcript_data = r#"{"type":"user","message":{"role":"user","content":"hello"},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":null}
invalid json line
{"type":"assistant","message":{"id":"msg_124","type":"message","role":"assistant","model":"test-model","content":"hi","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":1,"service_tier":"standard"}},"uuid":"test-uuid2","timestamp":"2025-01-01T00:00:01Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","requestId":"req-123"}
{"invalid": "missing type field"}
{"type":"summary","summary":"Test summary","leafUuid":"leaf-123"}"#;

    let result = parse_transcript_with_context(transcript_data);

    // Should have parsed 3 valid entries
    assert_eq!(result.entries.len(), 3);

    // Should have 2 errors
    assert_eq!(result.errors.len(), 2);

    // Check error line numbers
    assert_eq!(result.errors[0].line_number, 2); // "invalid json line"
    assert_eq!(result.errors[1].line_number, 4); // missing type field
}

#[test]
fn test_complex_content_structures() {
    // Test with complex content array
    let complex_json = r#"{"type":"assistant","message":{"id":"msg_125","type":"message","role":"assistant","model":"test-model","content":[{"type":"text","text":"Here's the code:"}],"thinking":"Let me write some code","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":1,"service_tier":"standard"}},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","requestId":"req-123"}"#;

    let entry = parse_transcript_line(complex_json).expect("Should parse complex content");

    match entry {
        TranscriptEntry::Assistant(assistant) => match &assistant.message {
            TranscriptMessage::Assistant {
                thinking, content, ..
            } => {
                assert!(thinking.is_some());
                assert!(content.is_some());
            }
            _ => panic!("Expected assistant message type"),
        },
        _ => panic!("Expected assistant entry"),
    }
}

#[test]
fn test_tool_uses_parsing() {
    // Test entry with tool uses
    let tool_use_json = r#"{"type":"assistant","message":{"id":"msg_126","type":"message","role":"assistant","model":"test-model","content":"I'll help you with that.","tool_uses":[{"toolName":"bash","toolInput":{"command":"ls"},"toolOutput":"file1.txt\nfile2.txt"}],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":1,"service_tier":"standard"}},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","requestId":"req-123"}"#;

    let entry = parse_transcript_line(tool_use_json).expect("Should parse tool use");

    match entry {
        TranscriptEntry::Assistant(assistant) => match &assistant.message {
            TranscriptMessage::Assistant { tool_uses, .. } => {
                assert!(tool_uses.is_some());
                let tool_uses = tool_uses.as_ref().unwrap();
                assert_eq!(tool_uses.len(), 1);
                assert_eq!(tool_uses[0].tool_name, "bash");
            }
            _ => panic!("Expected assistant message type"),
        },
        _ => panic!("Expected assistant entry"),
    }
}
