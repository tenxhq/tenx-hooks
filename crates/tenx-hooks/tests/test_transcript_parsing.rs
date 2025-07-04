use tenx_hooks::transcript::parse_transcript_line;

#[test]
fn test_tool_result_with_array_content() {
    let json_line = r#"{"parentUuid":"9a6212a9-b1d1-4823-b783-0c5248c2602c","isSidechain":false,"userType":"external","cwd":"/Users/cortesi/git/public/tenx-hooks/sandbox","sessionId":"bea16101-83bb-4150-b58d-295de3267bf9","version":"1.0.43","type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01UjUUPPebuG5bkbrpDPFTfQ","type":"tool_result","content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}]}]},"uuid":"c5c77671-8174-4149-abcc-650cc6058924","timestamp":"2025-07-04T10:59:44.274Z","toolUseResult":{"content":[{"type":"text","text":"I've successfully created the file `/Users/cortesi/git/public/tenx-hooks/sandbox/editme2.md` with the word \"test\" as its contents. Since the file didn't exist previously, I created it with just the word \"test\" as requested."}],"totalDurationMs":10185,"totalTokens":11766,"totalToolUseCount":2,"usage":{"input_tokens":7,"cache_creation_input_tokens":153,"cache_read_input_tokens":11543,"output_tokens":63,"service_tier":"standard"},"wasInterrupted":false}}"#;
    
    match parse_transcript_line(json_line) {
        Ok(entry) => {
            println!("Successfully parsed: {}", entry.description());
            assert_eq!(entry.entry_type, tenx_hooks::transcript::TranscriptEntryType::User);
        }
        Err(e) => {
            panic!("Failed to parse transcript: {}", e);
        }
    }
}