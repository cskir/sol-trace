use tokio_util::sync::CancellationToken;

pub struct SharedState {
    pub current_cancel: Option<CancellationToken>,
}

#[derive(Debug, Default)]
pub struct ScrollState {
    pub offset: u16,
    pub follow: bool,
}

#[derive(Debug, Default)]
pub enum Panel {
    Stream,
    History,
    #[default]
    Log,
    Repl,
}

#[derive(Debug, Default)]
pub struct LastHeights {
    pub stream: u16,
    pub history: u16,
    pub log: u16,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub stream_list: Vec<String>,
    pub history_list: Vec<String>,
    pub logs: Vec<String>,
    pub repl: String,

    pub stream_scroll: ScrollState,
    pub history_scroll: ScrollState,
    pub log_scroll: ScrollState,

    pub focused: Panel,
    pub last_heights: LastHeights,
}

impl AppState {
    pub fn update_heights(&mut self, stream: u16, history: u16, log: u16) {
        self.last_heights.stream = stream;
        self.last_heights.history = history;
        self.last_heights.log = log;
    }
}
