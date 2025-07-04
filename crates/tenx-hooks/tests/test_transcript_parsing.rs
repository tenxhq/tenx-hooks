use tenx_hooks::transcript::{
    ContentBlock, MessageContent, TranscriptEntry, TranscriptMessage, parse_transcript_line,
    parse_transcript_with_context,
};

#[test]
fn test_tool_result_with_array_content() {
    let json_line = r#"{"parentUuid":"9a6212a9-b1d1-4823-b783-0c5248c2602c","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx-hooks/sandbox","sessionId":"bea16101-83bb-4150-b58d-295de3267bf9","version":"1.0.43","type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01UjUUPPebuG5bkbrpDPFTfQ","type":"tool_result","content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}]}]},"uuid":"c5c77671-8174-4149-abcc-650cc6058924","timestamp":"2025-07-04T10:59:44.274Z","toolUseResult":{"content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}],"totalDurationMs":10185,"totalTokens":11766,"totalToolUseCount":2,"usage":{"input_tokens":7,"cache_creation_input_tokens":153,"cache_read_input_tokens":11543,"output_tokens":63,"service_tier":"standard"},"wasInterrupted":false}}"#;

    match parse_transcript_line(json_line) {
        Ok(entry) => {
            println!("Successfully parsed user entry");
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
    // Test system entry with toolUseID
    let system_json = r#"{"type":"system","uuid":"sys-uuid","timestamp":"2025-01-01T00:00:00Z","content":"test content","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","isMeta":false,"level":"info","toolUseID":"tool-123"}"#;
    let system_entry =
        parse_transcript_line(system_json).expect("Should parse system entry with toolUseID");
    assert!(matches!(system_entry, TranscriptEntry::System(_)));

    // Test system entry without toolUseID (like the error example)
    let system_json_no_tool = r#"{"type":"system","uuid":"sys-uuid","timestamp":"2025-01-01T00:00:00Z","content":"test content","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","isMeta":false,"level":"warning"}"#;
    let system_entry_no_tool = parse_transcript_line(system_json_no_tool)
        .expect("Should parse system entry without toolUseID");
    assert!(matches!(system_entry_no_tool, TranscriptEntry::System(_)));

    // Test user entry
    let user_json = r#"{"type":"user","message":{"role":"user","content":"hello"},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":null}"#;
    let user_entry = parse_transcript_line(user_json).expect("Should parse user entry");
    assert!(matches!(user_entry, TranscriptEntry::User(_)));

    // Test assistant entry with requestId
    let assistant_json = r#"{"type":"assistant","message":{"id":"msg_123","type":"message","role":"assistant","model":"test-model","content":"hello back","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":2,"service_tier":"standard"}},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123","requestId":"req-123"}"#;
    let assistant_entry =
        parse_transcript_line(assistant_json).expect("Should parse assistant entry with requestId");
    assert!(matches!(assistant_entry, TranscriptEntry::Assistant(_)));

    // Test assistant entry without requestId (like API error messages)
    let assistant_json_no_req = r#"{"type":"assistant","message":{"id":"msg_123","type":"message","role":"assistant","model":"test-model","content":"API error","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":2,"service_tier":"standard"}},"uuid":"test-uuid","timestamp":"2025-01-01T00:00:00Z","cwd":"/test","sessionId":"test-session","version":"1.0.0","userType":"external","isSidechain":false,"parentUuid":"parent-123"}"#;
    let assistant_entry_no_req = parse_transcript_line(assistant_json_no_req)
        .expect("Should parse assistant entry without requestId");
    assert!(matches!(
        assistant_entry_no_req,
        TranscriptEntry::Assistant(_)
    ));

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

#[test]
fn test_system_entry_without_tool_use_id() {
    // Test the specific error case from the user
    let json_line = r#"{"parentUuid":"e37aed7f-cdda-4333-8c92-00a414d22fc8","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/mcptool","sessionId":"781af6da-2cf7-4d2e-b727-c209ba16e259","version":"1.0.35","type":"system","content":"Claude Opus 4 limit reached, now using Sonnet 4","isMeta":false,"timestamp":"2025-06-28T03:14:56.367Z","uuid":"bd282060-4155-4b79-b87e-2c4e03902734","level":"warning"}"#;

    let entry =
        parse_transcript_line(json_line).expect("Should parse system entry without toolUseID");

    match entry {
        TranscriptEntry::System(system) => {
            assert_eq!(
                system.content,
                "Claude Opus 4 limit reached, now using Sonnet 4"
            );
            assert_eq!(system.level, Some("warning".to_string()));
            assert!(system.tool_use_id.is_none());
        }
        _ => panic!("Expected system entry"),
    }
}

#[test]
fn test_assistant_api_error_without_request_id() {
    // Test the specific API error case from the user
    let json_line = r#"{"parentUuid":"77e08109-515a-4538-96aa-0533e9e8919e","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/mcptool","sessionId":"781af6da-2cf7-4d2e-b727-c209ba16e259","version":"1.0.35","type":"assistant","uuid":"b2c32dfb-4626-470b-b313-c536efa975d5","timestamp":"2025-06-28T03:42:56.171Z","message":{"id":"a764c94f-cfc2-485d-b638-cd511a0da54e","model":"<synthetic>","role":"assistant","stop_reason":"stop_sequence","stop_sequence":"","type":"message","usage":{"input_tokens":0,"output_tokens":0,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"server_tool_use":{"web_search_requests":0},"service_tier":null},"content":[{"type":"text","text":"API Error: Request was aborted."}]},"isApiErrorMessage":true}"#;

    let entry = parse_transcript_line(json_line)
        .expect("Should parse assistant API error without requestId");

    match entry {
        TranscriptEntry::Assistant(assistant) => {
            assert!(assistant.request_id.is_none());
            assert_eq!(assistant.is_api_error_message, Some(true));
            // Verify it's an API error message
            match &assistant.message {
                TranscriptMessage::Assistant { content, .. } => {
                    assert!(content.is_some());
                    match content.as_ref().unwrap() {
                        MessageContent::Blocks(blocks) => {
                            assert_eq!(blocks.len(), 1);
                            match &blocks[0] {
                                ContentBlock::Text { text } => {
                                    assert!(text.contains("API Error"));
                                }
                                _ => panic!("Expected text block"),
                            }
                        }
                        _ => panic!("Expected blocks content"),
                    }
                }
                _ => panic!("Expected assistant message type"),
            }
        }
        _ => panic!("Expected assistant entry"),
    }
}

#[test]
fn test_assistant_thinking_content() {
    // Test the specific thinking content case from the user
    let json_line = r#"{"parentUuid":"b61aa179-3293-4026-9f56-3d3669621915","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/private/canopy","sessionId":"10e2a7a3-ff7d-413a-b4d4-b54fd440b090","version":"1.0.39","message":{"id":"msg_01Ha7YGwaRoKNPpeYig9SAr8","type":"message","role":"assistant","model":"claude-opus-4-20250514","content":[{"type":"thinking","thinking":"The user wants me to:\n1. Clean up the indenting with a helper function\n2. Include the node ID in the output\n\nLooking at the current dump.rs, I can see there's repetitive code for writing indented lines. I can create a helper function to handle the indentation and writing pattern. Also, I need to get the node ID and include it in the output.","signature":"EoAECkYIBRgCKkDGpUVzdwzTX0wl8BMZ+btk3WGIxcqPofkqRQSXTEvzT+OQbe5ukmDmTZljZoeudDXlxGalb3n28g+zv9gOCkCzEgzBeH8g0k2vlikw164aDCBLG+10q1rtCxCy/CIwW96aSkgqmOeUPHc7p3QIWub2ebnpWkg1njXRN3V3uA5S736H0I5aM/u2aMp6HZhfKucCpJvqJswdXjoNwX8KLMj2UmCdwNnLNWdqcQGGkhhqvdl8DUcwm3RRZo4phDWKLf0it3W/fEEpW8a6Qw75XG2cHmsjP2+727QTECoVnlhTRc2WbV7rmb8Zj6IjdNk924gdb9sgmBzIrY39cpEj1GrgEkDv+dgr2EK7n2j2HWcBGGyYtsZ2aUEGpv/MEvMDTU+zBFwvpUX/ROo05knnQ5s1xKxvkl2Pm06p6mABpqNos+50ez4P+Z3FkUNYBDaP1qgdTXX9cMSD+4KnULPrYmcI0HQW2MnA+6Den3Lgr5w0k18YU6TQopp6021lR04gFyRV/6N+MUCO0bQ5e3JvJUsgJu8ODyEBNV9jeyn8VIp1LeQR8TWtF0OmoxcYb5KjoDKRUDj4SogbUUydiyoYrrqW9KTUBGZ69jTfg3LnPkdsrS/KJ9Y1GftY0jrNrZs6xDyHjxDoRt8C/Tbaum+U9XCfEJjZRu7sz60YAQ=="}],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":10,"cache_creation_input_tokens":79478,"cache_read_input_tokens":13884,"output_tokens":7,"service_tier":"standard"}},"requestId":"req_011CQhJZyUbEVot2FFd6XPkn","type":"assistant","uuid":"097f1ac1-7017-4e6e-a415-041d4be97072","timestamp":"2025-07-02T01:12:52.249Z"}"#;

    let entry = parse_transcript_line(json_line)
        .expect("Should parse assistant entry with thinking content");

    match entry {
        TranscriptEntry::Assistant(assistant) => {
            assert!(assistant.request_id.is_some());
            // Verify thinking content
            match &assistant.message {
                TranscriptMessage::Assistant { content, .. } => {
                    assert!(content.is_some());
                    match content.as_ref().unwrap() {
                        MessageContent::Blocks(blocks) => {
                            assert_eq!(blocks.len(), 1);
                            match &blocks[0] {
                                ContentBlock::Thinking { thinking, .. } => {
                                    assert!(thinking.contains("Clean up the indenting"));
                                }
                                _ => panic!("Expected thinking block"),
                            }
                        }
                        _ => panic!("Expected blocks content"),
                    }
                }
                _ => panic!("Expected assistant message type"),
            }
        }
        _ => panic!("Expected assistant entry"),
    }
}

#[test]
fn test_system_entry_without_level() {
    // Test the specific system entry without level field
    let json_line = r#"{"parentUuid":"1f67ebc1-6080-4436-a4d1-fad302b74613","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx","sessionId":"cc2f4c22-5bac-4cf2-8136-a694786acbb6","version":"1.0.16","type":"system","content":"Claude Opus 4 limit reached, now using Sonnet 4","isMeta":false,"timestamp":"2025-06-06T03:42:49.574Z","uuid":"200b8f7f-327c-421b-839a-3045a77166c1"}"#;

    let entry = parse_transcript_line(json_line).expect("Should parse system entry without level");

    match entry {
        TranscriptEntry::System(system) => {
            assert_eq!(
                system.content,
                "Claude Opus 4 limit reached, now using Sonnet 4"
            );
            assert!(system.level.is_none());
            assert!(system.tool_use_id.is_none());
        }
        _ => panic!("Expected system entry"),
    }
}
