//! Pure JSON-RPC 2.0 envelope for the MCP server: parse an incoming request, build result/error
//! responses. The server mirror of the retired forge sidecar client's `jsonrpc_from_http`/`tool_text`
//! (#411 — that crate PARSED responses + BUILT requests as a client; this one PARSES requests +
//! BUILDS responses).

use serde_json::Value;

/// JSON-RPC / MCP error codes — the subset L1 emits.
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
/// Unknown method.
pub const METHOD_NOT_FOUND: i64 = -32601;
/// Bad params or unknown tool (MCP folds unknown-tool into invalid-params).
pub const INVALID_PARAMS: i64 = -32602;
/// MCP resource-not-found (spec §resources).
pub const RESOURCE_NOT_FOUND: i64 = -32002;

/// A parsed JSON-RPC message. A message with NO `id` is a notification (it gets no response).
#[derive(Debug, Clone, PartialEq)]
pub struct RpcRequest {
    /// The request id, or `None` for a notification.
    pub id: Option<Value>,
    /// The method name.
    pub method: String,
    /// The params object (or `Null` when absent).
    pub params: Value,
}

impl RpcRequest {
    /// The id to echo in a response, defaulting to `null` (a notification is never responded to, but a
    /// malformed request whose id we couldn't read still answers with `id: null`, per the spec).
    pub fn response_id(&self) -> Value {
        self.id.clone().unwrap_or(Value::Null)
    }

    /// Whether this message expects a response (a request has an id; a notification does not).
    pub fn is_request(&self) -> bool {
        self.id.is_some()
    }
}

/// Parse a single JSON-RPC message. `Err((code, message))` is a ready-to-send error tuple (the caller
/// answers with `id: null`), used only for a genuinely unparsable / non-object / method-less message.
pub fn parse_request(message: &str) -> Result<RpcRequest, (i64, String)> {
    let value: Value = serde_json::from_str(message)
        .map_err(|err| (PARSE_ERROR, format!("parse error: {err}")))?;
    let object = value
        .as_object()
        .ok_or((INVALID_REQUEST, "request is not a JSON object".to_string()))?;
    let method = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or((INVALID_REQUEST, "missing method".to_string()))?
        .to_string();
    // Absent id → notification (None); present (even null) → a request expecting a response.
    let id = object.get("id").cloned();
    let params = object.get("params").cloned().unwrap_or(Value::Null);
    Ok(RpcRequest { id, method, params })
}

/// Build a JSON-RPC success response for `id` carrying `result`.
pub fn result_response(id: &Value, result: Value) -> String {
    serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

/// Build a JSON-RPC error response for `id` with `code`/`message`.
pub fn error_response(id: &Value, code: i64, message: &str) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_reads_id_method_params() {
        let request =
            parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"a":2}}"#)
                .expect("valid");
        assert_eq!(request.method, "tools/list");
        assert!(request.is_request());
        assert_eq!(request.response_id(), serde_json::json!(1));
        assert_eq!(request.params, serde_json::json!({"a":2}));
    }

    #[test]
    fn parse_request_notification_has_no_id_and_null_params_default() {
        let request =
            parse_request(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).expect("ok");
        assert!(!request.is_request());
        assert_eq!(request.response_id(), Value::Null);
        assert_eq!(request.params, Value::Null);
    }

    #[test]
    fn parse_request_rejects_garbage_non_object_and_missing_method() {
        assert_eq!(
            parse_request("not json").expect_err("garbage").0,
            PARSE_ERROR
        );
        assert_eq!(
            parse_request("[1,2]").expect_err("array").0,
            INVALID_REQUEST
        );
        assert_eq!(
            parse_request(r#"{"id":1}"#).expect_err("no method").0,
            INVALID_REQUEST
        );
    }

    #[test]
    fn result_and_error_response_shapes() {
        let ok: Value = serde_json::from_str(&result_response(
            &serde_json::json!(7),
            serde_json::json!({"x":1}),
        ))
        .expect("json");
        assert_eq!(ok["jsonrpc"], "2.0");
        assert_eq!(ok["id"], 7);
        assert_eq!(ok["result"]["x"], 1);

        let err: Value =
            serde_json::from_str(&error_response(&Value::Null, METHOD_NOT_FOUND, "nope"))
                .expect("json");
        assert_eq!(err["jsonrpc"], "2.0");
        assert!(err["id"].is_null());
        assert_eq!(err["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(err["error"]["message"], "nope");
    }

    #[test]
    fn error_codes_are_the_json_rpc_spec_values() {
        // Pin the exact NEGATIVE values — a dropped sign (e.g. -32700 → 32700) is a real wire defect.
        assert_eq!(PARSE_ERROR, -32700);
        assert_eq!(INVALID_REQUEST, -32600);
        assert_eq!(METHOD_NOT_FOUND, -32601);
        assert_eq!(INVALID_PARAMS, -32602);
        assert_eq!(RESOURCE_NOT_FOUND, -32002);
    }
}
