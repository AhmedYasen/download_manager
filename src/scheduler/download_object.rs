use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Waiting,
    Active,
    Done,
    Failed,
}

impl Default for State {
    fn default() -> Self { State::Waiting }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            State::Waiting => "Waiting".to_string(),
            State::Active => "Active".to_string(),
            State::Done => "Done".to_string(),
            State::Failed => "Failed".to_string(),
        }
    }
}


// should be renamed to DownloadFileMetadata
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DownloadObject {
    pub name: String,
    pub download_path: String,
    pub url: String,
    pub state: State,
    pub total_size: Option<u64>,
    pub resulted_err: Option<String>,
}
