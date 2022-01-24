use super::download_object::*;
use crate::commands::{Manager, ManagerCommands};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::vec;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
};

use super::download_executor::DownloadExecutor;


/*
 * The Scheduler is the core of the download manager
 * - The most important function is the run function
 * - The scheduler has three lists [waiting, active, done]
 * - when calling the add function the download object is inserted in the waiting list
 * - if the running threads are less than the maximium jobs a download object moves to the active list
 * - if a running thread dies or finishes the download object moves to the done list
 * - the fourth list is the DownloadExecutor which creates a therad and hold the receiver end to the thread
 */



#[derive(Debug, Default)]
pub struct Scheduler {
    waiting_list: VecDeque<Arc<Mutex<DownloadObject>>>,
    active_list: HashMap<usize, Arc<Mutex<DownloadObject>>>,
    done_list: Vec<Arc<Mutex<DownloadObject>>>,
    download_executor: HashMap<usize, DownloadExecutor>,
    pub download_path: String,
    pub max_jobs: usize,
}

impl Scheduler {
    pub fn new(max_jobs: usize, download_path: String) -> Self {
        Scheduler {
            max_jobs,
            download_path,
            ..Default::default()
        }
    }

    // when calling the add function the download object is inserted in the waiting list
    pub fn add(
        &mut self,
        custom_name: Option<String>,
        download_path: String,
        url: String,
    ) -> anyhow::Result<()> {

        //set the name by the custom name the user set
        let name = {
            let mut fname = url.rsplit_once("/").unwrap().1.to_string();
            if let Some(cn) = custom_name {
                fname = format!("{}.{}", cn, fname.rsplit_once(".").unwrap().1);
            }
            fname
        };

        self.waiting_list
            .push_back(Arc::new(Mutex::new(DownloadObject {
                name,
                url,
                state: State::Waiting,
                total_size: None,
                download_path,
                resulted_err: None,
            })));

        Ok(())
    }

    // return a download object if the running threads less than the max_jobs
    fn next(&mut self) -> Option<Arc<Mutex<DownloadObject>>> {
        if self.active_list.len() < self.max_jobs {
            self.waiting_list.pop_front()
        } else {
            None
        }
    }

    // call a download executor to create a new thread and move a download object to the active list
    fn exec_in_thread(&mut self, id: usize, data: Arc<Mutex<DownloadObject>>) {
        let de = DownloadExecutor::new(id, Arc::clone(&data));
        self.active_list.insert(id, Arc::clone(&data));
        data.lock().unwrap().state = State::Active;

        self.download_executor.insert(id, de);
    }


    // if a thread finished the try_recv() will return the id of the thread
    // the id is sent when we call the download executor
    fn check_finished_threads(&mut self) {
        let mut to_be_removed = vec![];
        for (_, de) in &self.download_executor {
            if let Ok(id) = de.sched_rx.try_recv() {
                let down_done = self.active_list.remove(&id).unwrap();
                let _ = to_be_removed.push(id);
                self.done_list.push(down_done);
            }
        }

        for index in to_be_removed {
            self.download_executor.remove(&index);
        }
    }

    pub fn run(sched: Self, thread_rx: Receiver<Manager>, thread_tx: Sender<Vec<String>>) {
        let mut id_counter = 0_usize;
        let mut sched = sched;

        // this is the background thread that generates another threads for each download object
        std::thread::spawn(move || loop {
            // sched.debug_all();

            // if we can run a new download object => send it to a thread to handle
            if let Some(obj) = sched.next() {
                sched.exec_in_thread(id_counter, obj);
            }
            
            // loop on the try_recv() for each thread
            sched.check_finished_threads();
            id_counter += 1;


            // receives from the server
            if let Ok(mngr) = thread_rx.try_recv() {
                let mut list = vec![];

                // handling the commmands
                match mngr.subcommands {
                    ManagerCommands::Add {
                        url,
                        custom_name,
                        custom_download_path,
                    } => {

                        let down_path = if let Some(custom_download_path) = custom_download_path {
                            custom_download_path
                        } else {
                            sched.download_path.clone()
                        };
                        let ret = sched.add(custom_name, down_path, url);
                        if let Err(e) = ret {
                            list.push(e.to_string());
                        } else {
                            list.push(String::from("File added"));
                        }
                    }
                    ManagerCommands::List { subcommands } => match subcommands {
                        crate::commands::ListCommands::All => {
                            list.push(sched.stringify_waiting_list());
                            list.push(sched.stringify_active_list());
                            list.push(sched.stringify_done_list());
                        }
                        crate::commands::ListCommands::Active => {
                            list.push(sched.stringify_active_list());
                        }
                        crate::commands::ListCommands::Done => {
                            list.push(sched.stringify_done_list());
                        }
                    },
                    ManagerCommands::Info { filename } => {
                        if let Some(info) = sched.get_info(filename) {
                            list.push(info);
                        } else {
                            list.push(String::from("This file is not found!!"));
                        }
                    }
                    _ => (),
                }

                // send the result
                let _ = thread_tx.send(list);
            } else {
                // leave some space to the other threads
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    // fn debug_all(&self) {
    //     println!("waiting list: {:#?}\r\n", self.waiting_list.len());
    //     println!("active list: {:#?}\r\n", self.active_list.len());
    //     println!("done list: {:#?}\r\n", self.done_list.len());
    //     println!(
    //         "download exec list: {:#?}\r\n",
    //         self.download_executor.len()
    //     );
    //     println!("===============================\r\n\r\n");
    // }
}



// These are helper functions
impl Scheduler {
    // the stringify functions construct the files information in a string
    // 1 -> waiting
    fn stringify_waiting_list(&self) -> String {
        self.stringify_list(1)
    }
    // 2 -> active
    fn stringify_active_list(&self) -> String {
        self.stringify_list(2)
    }
    // 3 -> done
    fn stringify_done_list(&self) -> String {
        self.stringify_list(3)
    }
    fn stringify_list(&self, list_name: usize) -> String {
        let stringify_list = |list: &Arc<Mutex<DownloadObject>>| {
            let list_ptr = list.lock().unwrap();
            let mut inf = String::new();
            inf.push_str(list_ptr.name.as_str());
            inf.push_str("  ( _ / ");
            inf.push_str(list_ptr.total_size.unwrap().to_string().as_str());
            inf.push_str(")  ");
            inf.push_str(list_ptr.state.to_string().as_str());
            inf.push_str("\r\n");
            inf
        };
        let mut ret = String::new();
        match list_name {
            1 => {
                for list in &self.waiting_list {
                    ret.push_str(stringify_list(list).as_str());
                }
            }
            2 => {
                for (_, list) in &self.active_list {
                    ret.push_str(stringify_list(list).as_str());
                }
            }
            3 => {
                for list in &self.done_list {
                    ret.push_str(stringify_list(list).as_str());
                }
            }
            _ => ()
        }

        ret
    }


    //get information of a signle file
    fn get_info(&self, name: String) -> Option<String> {
        let check_list = |list: &Arc<Mutex<DownloadObject>>| {
            let list_ptr = list.lock().unwrap();
            let mut inf = String::new();
            if list_ptr.name.eq(&name) {
                inf.push_str(name.as_str());
                inf.push_str("  ( _ / ");
                inf.push_str(list_ptr.total_size.unwrap().to_string().as_str());
                inf.push_str(")  ");
                inf.push_str(list_ptr.state.to_string().as_str());
                inf.push_str("\r\n");
                return Some(inf);
            }

            None
        };

        for list in &self.done_list {
            let info = check_list(list);
            if info.is_some() {
                return info;
            }
        }
        for list in &self.waiting_list {
            let info = check_list(list);
            if info.is_some() {
                return info;
            }
        }
        for (_, list) in &self.active_list {
            let info = check_list(list);
            if info.is_some() {
                return info;
            }
        }

        None
    }

}
