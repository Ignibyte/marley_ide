//! The PURE request router (REQ-001/002/003/005/006/007/008/010): given a read-only [`RequestCtx`] + the
//! connection's mutable [`Subscriptions`], turn one JSON-RPC message into outbound messages + an optional
//! app-side effect. Never panics; every business refusal is a typed `isError` result (D6), every unknown
//! method/tool a protocol error.

use crate::jsonrpc::RpcRequest;
use crate::permission::Decision;
use crate::registry::Family;
use crate::{
    Effect, Handled, MCP_PROTOCOL_VERSION, Outgoing, RequestCtx, Subscriptions, jsonrpc,
    permission, registry, resource, tools,
};
use marley_fleet::SurfaceRequest;
use serde_json::{Value, json};

/// Handle one JSON-RPC message from a client. PURE — the transport shim runs the auth guards BEFORE this
/// and moves the returned [`Outgoing`] bytes + applies the [`Effect`] on the UI thread.
pub fn handle_message(ctx: &RequestCtx, subs: &mut Subscriptions, message: &str) -> Handled {
    let request = match jsonrpc::parse_request(message) {
        Ok(request) => request,
        Err((code, message)) => {
            return respond(jsonrpc::error_response(&Value::Null, code, &message));
        }
    };
    let id = request.response_id();
    match request.method.as_str() {
        "initialize" => respond(jsonrpc::result_response(&id, initialize_result())),
        "tools/list" => respond(jsonrpc::result_response(&id, registry::tools_list())),
        "resources/list" => respond(jsonrpc::result_response(&id, resource::resources_list())),
        "resources/read" => resources_read(ctx, &request, &id),
        "resources/subscribe" => resources_subscribe(subs, &request, &id),
        "tools/call" => tools_call(ctx, &request, &id),
        // An unknown REQUEST → protocol error; a NOTIFICATION (no id — including the
        // `notifications/initialized` lifecycle ack) is silently ignored, no response.
        other if request.is_request() => respond(jsonrpc::error_response(
            &id,
            jsonrpc::METHOD_NOT_FOUND,
            &format!("unknown method: {other}"),
        )),
        _ => Handled::default(),
    }
}

/// The outbound notifications for a snapshot change (REQ-003): a subscribed connection gets a
/// `notifications/resources/updated` for the fleet resource; an unsubscribed one gets nothing. This is the
/// L2-ready per-connection model; L1's single-resource transport treats an open SSE stream AS the
/// subscription (`transport::serve_sse_stream` feeds `fleet: true`), so the gate is exercised here at the
/// pure seam and is ready for the per-session store L2 adds.
pub fn snapshot_changed(subs: &Subscriptions) -> Vec<Outgoing> {
    if subs.fleet {
        vec![Outgoing::Notification(
            resource::resource_updated_notification(resource::FLEET_RESOURCE_URI),
        )]
    } else {
        Vec::new()
    }
}

/// A single-response result with no effect.
fn respond(response: String) -> Handled {
    Handled {
        outgoing: vec![Outgoing::Response(response)],
        effect: None,
    }
}

/// The `initialize` result — protocol version + the L1 capabilities (tools + subscribable resources).
fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": { "tools": {}, "resources": { "subscribe": true } },
        "serverInfo": { "name": "marley", "version": "0" }
    })
}

/// `resources/read` (D3): the fleet resource serves the snapshot serialization; an unknown uri → error.
fn resources_read(ctx: &RequestCtx, request: &RpcRequest, id: &Value) -> Handled {
    let uri = request
        .params
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match resource::resource_read(ctx.snapshot, uri) {
        Ok(result) => respond(jsonrpc::result_response(id, result)),
        Err((code, message)) => respond(jsonrpc::error_response(id, code, &message)),
    }
}

/// `resources/subscribe` (D3/REQ-003): subscribe the connection to the fleet resource; an unknown uri →
/// resource-not-found error (no subscription). (L1's transport treats an open SSE stream as the
/// subscription; this pure ack is the L2-ready per-session model.)
fn resources_subscribe(subs: &mut Subscriptions, request: &RpcRequest, id: &Value) -> Handled {
    let uri = request
        .params
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if uri == resource::FLEET_RESOURCE_URI {
        subs.fleet = true;
        respond(jsonrpc::result_response(id, json!({})))
    } else {
        respond(jsonrpc::error_response(
            id,
            jsonrpc::RESOURCE_NOT_FOUND,
            &format!("unknown resource: {uri}"),
        ))
    }
}

/// `tools/call` (REQ-002/004/005/006/007/008/010): unknown tool → protocol error; a denied permission →
/// `isError` refusal (D6); otherwise dispatch to the tool.
fn tools_call(ctx: &RequestCtx, request: &RpcRequest, id: &Value) -> Handled {
    let name = request
        .params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let arguments = request
        .params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Null);
    let Some(spec) = registry::lookup(name) else {
        // Unknown tool → protocol error (D6/REQ-010).
        return respond(jsonrpc::error_response(
            id,
            jsonrpc::INVALID_PARAMS,
            &format!("unknown tool: {name}"),
        ));
    };
    // Permission (D5/REQ-006/007/008). A DENIAL is a tool-execution error (isError), NOT a protocol error.
    if let Decision::Deny(reason) = permission::decide(spec.tier, spec.grant_class, ctx.grants) {
        return respond(jsonrpc::result_response(id, tools::tool_error(&reason)));
    }
    // Match on the family (EXHAUSTIVE — no catch-all; a new `Family` variant is a compile error until its
    // handler is wired, REQ-011). L1 has one tool per family.
    match spec.family {
        Family::Fleet => respond(jsonrpc::result_response(
            id,
            tools::fleet_snapshot_result(ctx.snapshot),
        )),
        Family::Session => surface_to_human(ctx, &arguments, id),
    }
}

/// `session.surface_to_human` (D7/REQ-004/005): resolve the id → a pane handle. Resolved → an `Accepted`
/// receipt + the focus effect for the app; unresolved → a `Refused` receipt (`isError`), NO effect, no
/// app-state change.
fn surface_to_human(ctx: &RequestCtx, arguments: &Value, id: &Value) -> Handled {
    let request: SurfaceRequest = match serde_json::from_value(arguments.clone()) {
        Ok(request) => request,
        Err(err) => {
            return respond(jsonrpc::result_response(
                id,
                tools::tool_error(&format!("invalid surface_to_human arguments: {err}")),
            ));
        }
    };
    match tools::resolve_surface(&request.id, ctx.surface_index) {
        Some(handle) => {
            let receipt = tools::surface_receipt(true, &request.id);
            Handled {
                outgoing: vec![Outgoing::Response(jsonrpc::result_response(
                    id,
                    tools::surface_result(&receipt),
                ))],
                effect: Some(Effect::SurfacePane(handle)),
            }
        }
        None => {
            let receipt = tools::surface_receipt(false, &request.id);
            respond(jsonrpc::result_response(
                id,
                tools::surface_result(&receipt),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_message, snapshot_changed};
    use crate::permission::GrantTable;
    use crate::{
        Effect, MCP_PROTOCOL_VERSION, Outgoing, RequestCtx, Subscriptions, jsonrpc, resource,
    };
    use marley_fleet::{FleetSnapshot, SessionEvent, State, reduce};

    fn snapshot() -> FleetSnapshot {
        reduce(
            FleetSnapshot::default(),
            &[SessionEvent::Upsert {
                id: "dev-1/a".into(),
                ts_ms: 1,
                title: "a".into(),
                state: State::Working,
                labels: Default::default(),
                transport: None,
            }],
        )
    }

    /// The JSON body of an outbound message. The or-pattern covers BOTH `Outgoing` variants, so the `let`
    /// is IRREFUTABLE — one always-executed arm, no uncovered discriminant/panic branch (both carry a body).
    fn body(outgoing: &Outgoing) -> serde_json::Value {
        let (Outgoing::Response(text) | Outgoing::Notification(text)) = outgoing;
        serde_json::from_str(text).expect("json")
    }

    /// The in-memory "fake transport" (S2): drive `handle_message` directly — no bound port.
    fn call(
        snap: &FleetSnapshot,
        grants: &GrantTable,
        index: &[(String, u64)],
        msg: &str,
    ) -> serde_json::Value {
        let ctx = RequestCtx {
            snapshot: snap,
            grants,
            surface_index: index,
        };
        let mut subs = Subscriptions::default();
        let handled = handle_message(&ctx, &mut subs, msg);
        body(handled.outgoing.first().expect("a response"))
    }

    #[test]
    fn initialize_advertises_the_l1_capabilities() {
        let r = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
        );
        assert_eq!(r["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert_eq!(r["result"]["capabilities"]["resources"]["subscribe"], true);
        assert!(r["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn tools_list_returns_two_tools() {
        let r = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
        );
        assert_eq!(r["result"]["tools"].as_array().expect("array").len(), 2);
    }

    #[test]
    fn fleet_snapshot_tool_returns_the_current_snapshot() {
        let r = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fleet.snapshot"}}"#,
        );
        assert_eq!(r["result"]["isError"], false);
        assert_eq!(
            r["result"]["structuredContent"]["seats"][0]["id"],
            "dev-1/a"
        );
    }

    #[test]
    fn unknown_method_and_unknown_tool_are_protocol_errors() {
        let m = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"bogus"}"#,
        );
        assert_eq!(m["error"]["code"], jsonrpc::METHOD_NOT_FOUND);
        let t = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nope.nope"}}"#,
        );
        assert_eq!(t["error"]["code"], jsonrpc::INVALID_PARAMS);
    }

    #[test]
    fn write_without_grant_is_iserror_not_a_protocol_error() {
        let r = call(
            &snapshot(),
            &GrantTable::default(), // no grant
            &[("dev-1/a".to_string(), 5)],
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"session.surface_to_human","arguments":{"id":"dev-1/a"}}}"#,
        );
        assert!(r.get("error").is_none()); // NOT a protocol error
        assert_eq!(r["result"]["isError"], true); // a tool-execution refusal (D6)
    }

    #[test]
    fn surface_resolved_emits_accepted_response_and_the_focus_effect() {
        let grants = GrantTable::from_classes(["session.write"]);
        let ctx = RequestCtx {
            snapshot: &snapshot(),
            grants: &grants,
            surface_index: &[("dev-1/a".to_string(), 42)],
        };
        let mut subs = Subscriptions::default();
        let handled = handle_message(
            &ctx,
            &mut subs,
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"session.surface_to_human","arguments":{"id":"dev-1/a"}}}"#,
        );
        assert_eq!(handled.effect, Some(Effect::SurfacePane(42)));
        assert!(matches!(handled.outgoing[0], Outgoing::Response(_)));
        let r = body(&handled.outgoing[0]);
        assert_eq!(r["result"]["isError"], false);
        assert_eq!(r["result"]["structuredContent"]["result"], "accepted");
    }

    #[test]
    fn surface_unknown_id_refuses_with_no_effect() {
        let grants = GrantTable::from_classes(["session.write"]);
        let ctx = RequestCtx {
            snapshot: &snapshot(),
            grants: &grants,
            surface_index: &[("dev-1/a".to_string(), 42)],
        };
        let mut subs = Subscriptions::default();
        let handled = handle_message(
            &ctx,
            &mut subs,
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"session.surface_to_human","arguments":{"id":"ghost"}}}"#,
        );
        assert_eq!(handled.effect, None); // REQ-005: no app-state change on an unknown id
        let r = body(&handled.outgoing[0]);
        assert_eq!(r["result"]["isError"], true);
    }

    #[test]
    fn resources_read_and_subscribe_and_unknown_uri() {
        let snap = snapshot();
        let read = call(
            &snap,
            &GrantTable::default(),
            &[],
            &format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{{"uri":"{}"}}}}"#,
                resource::FLEET_RESOURCE_URI
            ),
        );
        assert!(read["result"]["contents"][0]["text"].is_string());

        // resources/read of an UNKNOWN uri → a resource-not-found protocol error.
        let bad_read = call(
            &snap,
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"bad://x"}}"#,
        );
        assert_eq!(bad_read["error"]["code"], jsonrpc::RESOURCE_NOT_FOUND);

        let ctx = RequestCtx {
            snapshot: &snap,
            grants: &GrantTable::default(),
            surface_index: &[],
        };
        let mut subs = Subscriptions::default();
        handle_message(
            &ctx,
            &mut subs,
            &format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"resources/subscribe","params":{{"uri":"{}"}}}}"#,
                resource::FLEET_RESOURCE_URI
            ),
        );
        assert!(subs.fleet);

        let mut subs2 = Subscriptions::default();
        let bad = handle_message(
            &ctx,
            &mut subs2,
            r#"{"jsonrpc":"2.0","id":1,"method":"resources/subscribe","params":{"uri":"bad://x"}}"#,
        );
        assert!(!subs2.fleet);
        let r = body(&bad.outgoing[0]);
        assert_eq!(r["error"]["code"], jsonrpc::RESOURCE_NOT_FOUND);
    }

    #[test]
    fn resources_list_returns_the_fleet_resource() {
        let r = call(
            &snapshot(),
            &GrantTable::default(),
            &[],
            r#"{"jsonrpc":"2.0","id":1,"method":"resources/list"}"#,
        );
        assert_eq!(
            r["result"]["resources"][0]["uri"],
            resource::FLEET_RESOURCE_URI
        );
    }

    #[test]
    fn surface_bad_arguments_is_a_typed_iserror_refusal() {
        // `id` is a number, not a string → the SurfaceRequest parse fails → an isError refusal, no effect.
        let grants = GrantTable::from_classes(["session.write"]);
        let ctx = RequestCtx {
            snapshot: &snapshot(),
            grants: &grants,
            surface_index: &[],
        };
        let mut subs = Subscriptions::default();
        let handled = handle_message(
            &ctx,
            &mut subs,
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"session.surface_to_human","arguments":{"id":123}}}"#,
        );
        assert_eq!(handled.effect, None);
        assert_eq!(body(&handled.outgoing[0])["result"]["isError"], true);
    }

    #[test]
    fn notifications_get_no_response_parse_error_does() {
        let ctx = RequestCtx {
            snapshot: &snapshot(),
            grants: &GrantTable::default(),
            surface_index: &[],
        };
        let mut subs = Subscriptions::default();
        let init = handle_message(
            &ctx,
            &mut subs,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        );
        assert!(init.outgoing.is_empty() && init.effect.is_none());
        let unknown = handle_message(
            &ctx,
            &mut subs,
            r#"{"jsonrpc":"2.0","method":"some/notification"}"#,
        );
        assert!(unknown.outgoing.is_empty());
        let parse = call(&snapshot(), &GrantTable::default(), &[], "garbage");
        assert_eq!(parse["error"]["code"], jsonrpc::PARSE_ERROR);
        assert!(parse["id"].is_null());
    }

    #[test]
    fn snapshot_changed_gates_on_subscription() {
        assert_eq!(snapshot_changed(&Subscriptions { fleet: true }).len(), 1);
        assert!(snapshot_changed(&Subscriptions::default()).is_empty());
        let notifications = snapshot_changed(&Subscriptions { fleet: true });
        assert!(matches!(notifications[0], Outgoing::Notification(_)));
        let n = body(&notifications[0]);
        assert_eq!(n["params"]["uri"], resource::FLEET_RESOURCE_URI);
    }
}
