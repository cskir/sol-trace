use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LogSubscribeWsMessage {
    Subscribed(LogSubscribeResponse),
    Notification(LogsNotification),
    UnSubscribed(LogUnsubscribeResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Deserialize)]
pub struct LogSubscribeResponse {
    pub result: u64,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogsNotification {
    pub method: String,
    pub params: LogsParams,
}

#[derive(Debug, Deserialize)]
pub struct LogsParams {
    pub result: LogsResult,
    pub subscription: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogsResult {
    pub context: Context,
    pub value: LogsValue,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogsValue {
    pub signature: String,
    #[serde(default)]
    pub err: Option<serde_json::Value>,
    //pub logs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogUnsubscribeResponse {
    pub result: bool,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: RpcError,
    pub id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}
