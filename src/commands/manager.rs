use super::manager_commands::*;
use anyhow::Result;
use structopt::StructOpt;
// use crate::server::server_main;
use reqwest::blocking::Client;
use crate::server::main_thread;
use serde::{Serialize, Deserialize};


#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
#[structopt(
    about = "Download manager v1.0 <ahmedyasen8@gmail.com>",
    name = "manager"
)]
pub struct Manager {
    #[structopt(subcommand)]
    pub subcommands: ManagerCommands,
}

impl Manager {
    pub fn handle(&self) -> Result<String> {
        match self.subcommands.clone() {
            ManagerCommands::Start {
                active_downloads,
                download_path,
            } => {
                main_thread(active_downloads, download_path);
                Ok(String::from("Good Bye!"))
            }
            ManagerCommands::Cancel { .. } => {
                Ok(String::from("cancel command [not working yet]"))
            }
            _ => {
                let client = Client::new();
                let resp = client.post("http://127.0.0.1:7878/command").json(&self.clone()).send()?;

                let resp = resp.text()?;

                Ok(resp)

            }
        }
        
    }
}
