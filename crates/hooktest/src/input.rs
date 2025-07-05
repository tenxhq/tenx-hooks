use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Parse key=value pairs into a HashMap of string values
pub fn parse_string_inputs(inputs: &[String]) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();

    for input in inputs {
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid input format '{}'. Expected 'key=value'",
                input
            ));
        }

        let key = parts[0].to_string();
        let value = Value::String(parts[1].to_string());
        map.insert(key, value);
    }

    Ok(map)
}

/// Parse key=json pairs into a HashMap of JSON values
pub fn parse_json_inputs(inputs: &[String]) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();

    for input in inputs {
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid input format '{}'. Expected 'key=json'",
                input
            ));
        }

        let key = parts[0].to_string();
        let value: Value = serde_json::from_str(parts[1])
            .with_context(|| format!("Failed to parse JSON for key '{}': {}", key, parts[1]))?;
        map.insert(key, value);
    }

    Ok(map)
}

/// Combine multiple input sources into a single HashMap
/// Priority: json inputs override string inputs, both override base
pub fn combine_inputs(
    base: Option<HashMap<String, Value>>,
    string_inputs: &[String],
    json_inputs: &[String],
) -> Result<HashMap<String, Value>> {
    let mut result = base.unwrap_or_default();

    // Add string inputs
    let string_map = parse_string_inputs(string_inputs)?;
    for (key, value) in string_map {
        result.insert(key, value);
    }

    // Add JSON inputs (these override string inputs)
    let json_map = parse_json_inputs(json_inputs)?;
    for (key, value) in json_map {
        result.insert(key, value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_inputs() {
        let inputs = vec![
            "command=echo hello".to_string(),
            "file=/tmp/test.txt".to_string(),
        ];

        let result = parse_string_inputs(&inputs).unwrap();
        assert_eq!(
            result.get("command").unwrap(),
            &Value::String("echo hello".to_string())
        );
        assert_eq!(
            result.get("file").unwrap(),
            &Value::String("/tmp/test.txt".to_string())
        );
    }

    #[test]
    fn test_parse_json_inputs() {
        let inputs = vec![
            r#"args=["one", "two", "three"]"#.to_string(),
            r#"count=42"#.to_string(),
            r#"config={"debug": true}"#.to_string(),
        ];

        let result = parse_json_inputs(&inputs).unwrap();
        assert_eq!(
            result.get("args").unwrap(),
            &serde_json::json!(["one", "two", "three"])
        );
        assert_eq!(result.get("count").unwrap(), &Value::Number(42.into()));
        assert_eq!(
            result.get("config").unwrap(),
            &serde_json::json!({"debug": true})
        );
    }

    #[test]
    fn test_combine_inputs() {
        let mut base = HashMap::new();
        base.insert("existing".to_string(), Value::String("base".to_string()));

        let string_inputs = vec!["command=test".to_string()];
        let json_inputs = vec![r#"command="override""#.to_string()];

        let result = combine_inputs(Some(base), &string_inputs, &json_inputs).unwrap();

        // JSON input should override string input
        assert_eq!(
            result.get("command").unwrap(),
            &Value::String("override".to_string())
        );
        // Base value should remain
        assert_eq!(
            result.get("existing").unwrap(),
            &Value::String("base".to_string())
        );
    }
}
