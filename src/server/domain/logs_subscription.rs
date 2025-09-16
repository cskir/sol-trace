use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogsNotification {
    pub jsonrpc: String,
    pub method: String,
    pub(crate) params: LogsParams,
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
    pub logs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogSubscribeResponse {
    pub jsonrpc: String,
    pub result: u64,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogUnsubscribeResponse {
    pub jsonrpc: String,
    pub result: bool,
    pub id: u64,
}
