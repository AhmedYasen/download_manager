use std::sync::mpsc;



pub struct AppData<S, R> {
    pub server_tx: mpsc::Sender<S>,
    pub server_rx: mpsc::Receiver<R>,
    pub download_path: String,
}