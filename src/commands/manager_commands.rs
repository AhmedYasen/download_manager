
use structopt::StructOpt;
use serde::{Serialize, Deserialize};

#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
pub enum ManagerCommands {
    /// Add a url with optional name to be downloaded, name will be file name by default
    Add {
        #[structopt(short, long)]
        url: String,
        #[structopt(short="f", long)]
        custom_name: Option<String>,
        #[structopt(short="p", long)]
        custom_download_path: Option<String>,
    },
    /// List [all, active, done] jobs
    List {
        #[structopt(subcommand)]
        subcommands: ListCommands,
    },
    /// Cancel an active job
    Cancel {
        #[structopt(short)]
        filename: String,
    },
    /// Prints info for a job
    Info {
        #[structopt(short)]
        filename: String,
    },
    /// starts the program
    Start {
        #[structopt(short, long)]
        active_downloads: u16,
        #[structopt(short="p", long)]
        download_path: String,
    }
}

#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
pub enum ListCommands {
    All,
    Active,
    Done,
}