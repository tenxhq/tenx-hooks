use tenx_hooks::transcript::{
    TranscriptEntryWithRaw, find_missing_fields, parse_transcript_line, parse_transcript_with_raw,
    validate_transcript_entry,
};

#[test]
fn test_tool_result_with_array_content() {
    let json_line = r#"{"parentUuid":"9a6212a9-b1d1-4823-b783-0c5248c2602c","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx-hooks/sandbox","sessionId":"bea16101-83bb-4150-b58d-295de3267bf9","version":"1.0.43","type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01UjUUPPebuG5bkbrpDPFTfQ","type":"tool_result","content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}]}]},"uuid":"c5c77671-8174-4149-abcc-650cc6058924","timestamp":"2025-07-04T10:59:44.274Z","toolUseResult":{"content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}],"totalDurationMs":10185,"totalTokens":11766,"totalToolUseCount":2,"usage":{"input_tokens":7,"cache_creation_input_tokens":153,"cache_read_input_tokens":11543,"output_tokens":63,"service_tier":"standard"},"wasInterrupted":false}}"#;

    match parse_transcript_line(json_line) {
        Ok(entry) => {
            println!("Successfully parsed: {}", entry.description());
            assert_eq!(
                entry.entry_type,
                tenx_hooks::transcript::TranscriptEntryType::User
            );
        }
        Err(e) => {
            panic!("Failed to parse transcript: {e}");
        }
    }
}

#[test]
fn test_find_missing_fields() {
    // Test case 1: Identical objects
    let raw = serde_json::json!({
        "type": "user",
        "message": {
            "content": "hello"
        }
    });
    let parsed = raw.clone();
    let missing = find_missing_fields(&raw, &parsed, &[]);
    assert!(missing.is_empty());

    // Test case 2: Missing field at root level
    let raw = serde_json::json!({
        "type": "user",
        "unknownField": "value",
        "message": {
            "content": "hello"
        }
    });
    let parsed = serde_json::json!({
        "type": "user",
        "message": {
            "content": "hello"
        }
    });
    let missing = find_missing_fields(&raw, &parsed, &[]);
    assert_eq!(missing, vec!["unknownField"]);

    // Test case 3: Missing nested field
    let raw = serde_json::json!({
        "type": "user",
        "message": {
            "content": "hello",
            "unknownNested": "value"
        }
    });
    let parsed = serde_json::json!({
        "type": "user",
        "message": {
            "content": "hello"
        }
    });
    let missing = find_missing_fields(&raw, &parsed, &[]);
    assert_eq!(missing, vec!["message.unknownNested"]);

    // Test case 4: Arrays with missing fields
    let raw = serde_json::json!({
        "items": [
            {"id": 1, "name": "item1", "extra": "field"},
            {"id": 2, "name": "item2"}
        ]
    });
    let parsed = serde_json::json!({
        "items": [
            {"id": 1, "name": "item1"},
            {"id": 2, "name": "item2"}
        ]
    });
    let missing = find_missing_fields(&raw, &parsed, &[]);
    assert_eq!(missing, vec!["items.[0].extra"]);
}

#[test]
fn test_validate_transcript_entry() {
    // Create a transcript entry with extra fields in the raw JSON
    let raw_json = r#"{"type":"system","subtype":"init","sessionId":"test-session","timestamp":"2025-01-01T00:00:00Z","unknownField":"someValue","nestedObject":{"key":"value"}}"#;

    let entry = parse_transcript_line(raw_json).expect("Should parse");
    let entry_with_raw = TranscriptEntryWithRaw {
        entry,
        raw: raw_json.to_string(),
    };

    let missing_fields =
        validate_transcript_entry(&entry_with_raw).expect("Validation should succeed");

    // Should find the unknown fields
    assert!(missing_fields.contains(&"unknownField".to_string()));
    assert!(missing_fields.contains(&"nestedObject".to_string()));
}

#[test]
fn test_parse_transcript_with_raw() {
    let transcript_content = r#"{"type":"system","subtype":"init","sessionId":"test-session","timestamp":"2025-01-01T00:00:00Z"}
{"type":"user","message":{"content":"Hello"}}
{"type":"assistant","message":{"content":"Hi there"}}"#;

    let result = parse_transcript_with_raw(transcript_content);

    assert_eq!(result.entries.len(), 3);
    assert!(result.errors.is_empty());

    // Check that raw JSON is preserved
    assert!(result.entries[0].raw.contains("\"type\":\"system\""));
    assert!(result.entries[1].raw.contains("\"type\":\"user\""));
    assert!(result.entries[2].raw.contains("\"type\":\"assistant\""));
}

#[test]
fn test_strict_validation_detects_extra_fields() {
    // Test with the actual complex JSON from the first test
    let json_line = r#"{"parentUuid":"9a6212a9-b1d1-4823-b783-0c5248c2602c","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx-hooks/sandbox","sessionId":"bea16101-83bb-4150-b58d-295de3267bf9","version":"1.0.43","type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01UjUUPPebuG5bkbrpDPFTfQ","type":"tool_result","content":[{"type":"text","text":"Test text"}]}]},"uuid":"c5c77671-8174-4149-abcc-650cc6058924","timestamp":"2025-07-04T10:59:44.274Z","toolUseResult":{"content":[{"type":"text","text":"Test text"}],"totalDurationMs":10185,"totalTokens":11766,"totalToolUseCount":2,"usage":{"input_tokens":7,"cache_creation_input_tokens":153,"cache_read_input_tokens":11543,"output_tokens":63,"service_tier":"standard"},"wasInterrupted":false}}"#;

    let entry = parse_transcript_line(json_line).expect("Should parse");
    let entry_with_raw = TranscriptEntryWithRaw {
        entry,
        raw: json_line.to_string(),
    };

    let missing_fields =
        validate_transcript_entry(&entry_with_raw).expect("Validation should succeed");

    // These fields should be detected as missing from TranscriptEntry
    let expected_missing = vec![
        "parentUuid",
        "isSidechain",
        "userType",
        "cwd",
        "version",
        "uuid",
        "toolUseResult",
        "message.role",
    ];

    for field in expected_missing {
        assert!(
            missing_fields.iter().any(|f| f.contains(field)),
            "Expected to find missing field: {field}, but got: {missing_fields:?}"
        );
    }
}

#[test]
fn test_strict_validation_with_summary_entry() {
    // Test the exact example from the user
    let json_line = r#"{"type":"summary","summary":"Hacker News Story Title Scraped to Markdown File","leafUuid":"6cd0df94-36ea-470d-b09d-dac35ecad626"}"#;

    let entry = parse_transcript_line(json_line).expect("Should parse");
    let entry_with_raw = TranscriptEntryWithRaw {
        entry,
        raw: json_line.to_string(),
    };

    let missing_fields =
        validate_transcript_entry(&entry_with_raw).expect("Validation should succeed");

    // These fields should be detected as missing from TranscriptEntry
    assert!(
        missing_fields.contains(&"summary".to_string()),
        "Should detect missing 'summary' field, got: {missing_fields:?}"
    );
    assert!(
        missing_fields.contains(&"leafUuid".to_string()),
        "Should detect missing 'leafUuid' field, got: {missing_fields:?}"
    );
}

#[test]
fn test_parse_transcript_with_raw_validation() {
    // Test multiple entries with various unknown fields
    let transcript_content = r#"{"type":"summary","summary":"Test Summary","leafUuid":"6cd0df94-36ea-470d-b09d-dac35ecad626"}
{"type":"system","subtype":"init","sessionId":"test-session","timestamp":"2025-01-01T00:00:00Z","unknownSystemField":"ignored"}
{"type":"user","message":{"content":"Hello","unknownMessageField":"value"},"customField":"custom"}
{"type":"assistant","message":{"content":"Hi there"},"modelVersion":"v2","extraData":{"key":"value"}}"#;

    let result = parse_transcript_with_raw(transcript_content);

    assert_eq!(result.entries.len(), 4);
    assert!(result.errors.is_empty());

    // Validate each entry
    let validations: Vec<_> = result
        .entries
        .iter()
        .map(|entry| validate_transcript_entry(entry).expect("Validation should succeed"))
        .collect();

    // First entry (summary) should have missing fields
    assert!(validations[0].contains(&"summary".to_string()));
    assert!(validations[0].contains(&"leafUuid".to_string()));

    // Second entry (system) should have missing fields
    assert!(validations[1].contains(&"unknownSystemField".to_string()));

    // Third entry (user) should have missing fields
    assert!(validations[2].contains(&"customField".to_string()));
    assert!(validations[2].contains(&"message.unknownMessageField".to_string()));

    // Fourth entry (assistant) should have missing fields
    assert!(validations[3].contains(&"modelVersion".to_string()));
    assert!(validations[3].contains(&"extraData".to_string()));
}

#[test]
fn test_strict_validation_with_known_fields_only() {
    // Test that entries with only known fields pass strict validation
    let transcript_content = r#"{"type":"system","subtype":"init","sessionId":"test-session","timestamp":"2025-01-01T00:00:00Z"}
{"type":"user","message":{"content":"Hello"}}
{"type":"assistant","message":{"content":"Hi there","thinking":"Internal thought"}}
{"type":"result","status":"success","duration":1.5,"tokenUsage":{"inputTokens":10,"outputTokens":20}}"#;

    let result = parse_transcript_with_raw(transcript_content);

    assert_eq!(result.entries.len(), 4);
    assert!(result.errors.is_empty());

    // All entries should pass validation (no missing fields)
    for entry in &result.entries {
        let missing_fields = validate_transcript_entry(entry).expect("Validation should succeed");
        assert!(
            missing_fields.is_empty(),
            "Entry should have no missing fields, but found: {missing_fields:?}"
        );
    }
}
