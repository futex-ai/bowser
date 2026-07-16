//! Raw DevTools helpers for iframe targets that Chromiumoxide does not initialize as pages.

use std::collections::HashMap;

use async_tungstenite::tokio::ConnectStream;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::{WebSocketStream, tokio::connect_async};
use chromiumoxide::cdp::browser_protocol::page::FrameId;
use futures::StreamExt;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::cdp_trace;
use crate::error::{Error, Result};

pub(super) struct RemoteFrameTarget {
    socket: WebSocketStream<ConnectStream>,
    next_id: u64,
}

pub(super) struct FrameTreeSnapshot {
    pub(super) ordered_frames: Vec<FrameId>,
    pub(super) parents: HashMap<FrameId, FrameId>,
}

#[derive(Deserialize)]
struct FrameTreeResponse {
    #[serde(rename = "frameTree")]
    frame_tree: FrameTreeNode,
}

#[derive(Deserialize)]
struct FrameTreeNode {
    frame: FrameTreeFrame,
    #[serde(rename = "childFrames", default)]
    child_frames: Vec<FrameTreeNode>,
}

#[derive(Deserialize)]
struct FrameTreeFrame {
    id: FrameId,
}

#[derive(Deserialize)]
struct RemoteEvaluateResponse {
    result: RemoteValue,
}

#[derive(Deserialize)]
struct RemoteResolveNodeResponse {
    object: RemoteObjectIdValue,
}

#[derive(Deserialize)]
struct RemoteValue {
    value: Option<Value>,
}

#[derive(Deserialize)]
struct RemoteObjectIdValue {
    #[serde(rename = "objectId")]
    object_id: Option<String>,
}

#[derive(Deserialize)]
struct RemoteFrameOwnerResponse {
    #[serde(rename = "backendNodeId")]
    backend_node_id: i64,
}

impl RemoteFrameTarget {
    pub(super) async fn connect(websocket_url: &str) -> Result<Self> {
        let (socket, _) =
            connect_async(websocket_url)
                .await
                .map_err(|err| Error::JsEvaluation {
                    reason: err.to_string(),
                })?;
        Ok(Self { socket, next_id: 1 })
    }

    pub(super) async fn ordered_frames(&mut self) -> Result<FrameTreeSnapshot> {
        let response: FrameTreeResponse = self.call("Page.getFrameTree", json!({})).await?;
        let mut ordered_frames = Vec::new();
        let mut parents = HashMap::new();
        flatten_frame_tree(
            &response.frame_tree,
            None,
            0,
            &mut ordered_frames,
            &mut parents,
        );
        ordered_frames.sort_by_key(|(_, depth)| *depth);
        Ok(FrameTreeSnapshot {
            ordered_frames: ordered_frames
                .into_iter()
                .map(|(frame_id, _)| frame_id)
                .collect(),
            parents,
        })
    }

    pub(super) async fn evaluate_frame_value<T: DeserializeOwned>(
        &mut self,
        frame_id: &FrameId,
        expression: String,
    ) -> Result<T> {
        let context_id = self.create_isolated_world(frame_id).await?;
        let response: RemoteEvaluateResponse = self
            .call(
                "Runtime.evaluate",
                json!({
                    "expression": expression,
                    "contextId": context_id,
                    "returnByValue": true,
                    "awaitPromise": true,
                }),
            )
            .await?;
        let value = response.result.value.ok_or(Error::CaptureParse)?;
        serde_json::from_value(value).map_err(|_| Error::CaptureParse)
    }

    pub(super) async fn owner_temp_id(
        &mut self,
        owner_frame_id: &FrameId,
        child_frame_id: &FrameId,
    ) -> Result<Option<u32>> {
        let owner: RemoteFrameOwnerResponse = self
            .call(
                "DOM.getFrameOwner",
                json!({ "frameId": child_frame_id.as_ref() }),
            )
            .await?;
        let context_id = self.create_isolated_world(owner_frame_id).await?;
        let resolved: RemoteResolveNodeResponse = self
            .call(
                "DOM.resolveNode",
                json!({
                    "backendNodeId": owner.backend_node_id,
                    "executionContextId": context_id,
                }),
            )
            .await?;
        let Some(object_id) = resolved.object.object_id else {
            return Ok(None);
        };
        let response: RemoteEvaluateResponse = self
            .call(
                "Runtime.callFunctionOn",
                json!({
                    "functionDeclaration": "function() { return this.__bowserCaptureId ?? null; }",
                    "objectId": object_id,
                    "returnByValue": true,
                }),
            )
            .await?;
        Ok(response
            .result
            .value
            .and_then(|value| value.as_u64())
            .and_then(|value| u32::try_from(value).ok()))
    }

    pub(super) async fn dispatch_mouse_event(
        &mut self,
        event_type: &str,
        x: f64,
        y: f64,
        clicked: bool,
    ) -> Result<()> {
        let mut params = json!({
            "type": event_type,
            "x": x,
            "y": y,
        });
        if clicked {
            params["button"] = Value::from("left");
            params["clickCount"] = Value::from(1);
        }
        let _: Value = self.call("Input.dispatchMouseEvent", params).await?;
        Ok(())
    }

    async fn create_isolated_world(&mut self, frame_id: &FrameId) -> Result<i64> {
        #[derive(Deserialize)]
        struct CreateWorldResponse {
            #[serde(rename = "executionContextId")]
            execution_context_id: i64,
        }

        let response: CreateWorldResponse = self
            .call(
                "Page.createIsolatedWorld",
                json!({
                    "frameId": frame_id.as_ref(),
                    "worldName": "__bowser_capture__",
                }),
            )
            .await?;
        Ok(response.execution_context_id)
    }

    pub(super) async fn call<T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<T> {
        cdp_trace::record_method(method);
        let id = self.next_id;
        self.next_id += 1;
        self.socket
            .send(Message::Text(
                json!({ "id": id, "method": method, "params": params })
                    .to_string()
                    .into(),
            ))
            .await
            .map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;

        loop {
            let Some(message) = self.socket.next().await else {
                return Err(Error::BrowserDisconnected);
            };
            let message = message.map_err(|err| Error::JsEvaluation {
                reason: err.to_string(),
            })?;
            let Message::Text(text) = message else {
                continue;
            };
            let payload: Value =
                serde_json::from_str(text.as_ref()).map_err(|err| Error::JsEvaluation {
                    reason: err.to_string(),
                })?;
            if payload.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = payload.get("error") {
                return Err(Error::JsEvaluation {
                    reason: error.to_string(),
                });
            }
            let result = payload.get("result").cloned().ok_or(Error::CaptureParse)?;
            return serde_json::from_value(result).map_err(|_| Error::CaptureParse);
        }
    }
}

fn flatten_frame_tree(
    node: &FrameTreeNode,
    parent_frame_id: Option<&FrameId>,
    depth: usize,
    ordered_frames: &mut Vec<(FrameId, usize)>,
    parents: &mut HashMap<FrameId, FrameId>,
) {
    ordered_frames.push((node.frame.id.clone(), depth));
    if let Some(parent_frame_id) = parent_frame_id {
        parents.insert(node.frame.id.clone(), parent_frame_id.clone());
    }
    for child in &node.child_frames {
        flatten_frame_tree(
            child,
            Some(&node.frame.id),
            depth + 1,
            ordered_frames,
            parents,
        );
    }
}
