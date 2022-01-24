use super::download_object::*;
use chrono::Utc;
use reqwest::blocking::Client;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{
    sync::mpsc::{self, Receiver},
    sync::{Arc, Mutex},
    thread,
};

type Msg = String;

/*
 * The DownloadExecutor's job is to take a reference to a download object and run it in a thread 
 * 
 * - When the file finish to download the thread die
 * - Before the thread die, it send the id of the active download_object to move it to done either it Done or it Failed 
*/


#[derive(Debug)]
pub struct DownloadExecutor {
    pub sched_rx: Receiver<usize>,
}

impl DownloadExecutor {
    pub fn new(id: usize, download_obj: Arc<Mutex<DownloadObject>>) -> Self {
        let (thread_tx, sched_rx) = mpsc::channel();

        thread::spawn(move || {
            // creates client 
            let client = Client::new();

            // extract some values insted locking the mutex all the time to use the values
            let (url, download_path, mut name) = {
                let down_obj_ptr = Arc::clone(&download_obj);
                let down_obj_ptr = down_obj_ptr.lock().unwrap();
                (
                    down_obj_ptr.url.clone(),
                    down_obj_ptr.download_path.clone(),
                    down_obj_ptr.name.clone(),
                )
            };

            // closure to send that the task id is failed
            let send_failed = |e: String| {
                // updating the download metadata
                let down_obj_ptr = Arc::clone(&download_obj);
                let mut down_obj_ptr = down_obj_ptr.lock().unwrap();
                down_obj_ptr.resulted_err = Some(e.clone());
                down_obj_ptr.state = State::Failed;
                down_obj_ptr.resulted_err = Some(e);
                thread_tx.send(id).unwrap();
            };

            // check if file can be downloaded and update the total file size
            match Self::does_it_can_be_downloaded(url.as_str(), &client) {
                Ok(info) => {
                    Arc::clone(&download_obj).lock().unwrap().total_size = info;
                }
                Err(e) => {
                    send_failed(e);
                    return;
                }
            }

            // download the file
            let download_result =
                Self::download_the_file(url.as_str(), &client, download_path.as_str(), &mut name);

            if let Err(e) = download_result {
                send_failed(e);
                return;
            }


            // update the download object metadata
            let down_obj_ptr = Arc::clone(&download_obj);
            let mut down_obj_ptr = down_obj_ptr.lock().unwrap();

            down_obj_ptr.state = State::Done;
            down_obj_ptr.name = download_result.unwrap().unwrap();

            // flag the main thread
            thread_tx.send(id).unwrap();
        });

        Self { sched_rx }
    }

    // send a head request and if there is a response then the file can be downloaded
    // if it can be downloaded the function return the total_size of the file
    // if not return erorr message
    fn does_it_can_be_downloaded(url: &str, client: &Client) -> Result<Option<u64>, Msg> {

        // result will be ok if the file can be downloaded
        let file_info = client.head(url).send();
        match file_info {
            Ok(info) => {
                let headers = info.headers();
                if headers.contains_key("content-length") {
                    // If content-length is found => no problem for the unwrap()
                    return Ok(Some(
                        headers
                            .get("content-length")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .parse::<usize>()
                            .unwrap() as u64,
                    ));
                }

                Ok(None)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    // This function download the file and save it
    // again the function doesn't return the file, it stores it directly
    // if the file stored well the function return the filename
    fn download_the_file(
        url: &str,
        client: &Client,
        download_path: &str,
        file_name: &mut String,
    ) -> Result<Option<String>, Msg> {

        // download the file
        let downloaded_file = client.get(url).send().map_err(|e| e.to_string())?;
        let (mut file, file_name) = Self::safely_open_the_file(download_path, file_name);

        //store it
        let _written = file
            .write(&downloaded_file.bytes().map_err(|_e| String::from("Encoding Error: Failed to save the file"))?)
            .map_err(|e| e.to_string())?;

        // return the final file_name
        Ok(Some(file_name))
    }


    // safely because if the download_path is not exist the function will create it for the user
    // and if the filename exists it will append a time stamp to the file_name to make the it unique
    fn safely_open_the_file(download_path: &str, file_name: &str) -> (File, String) {

        // creates all non-exists directories
        let _ = std::fs::create_dir_all(download_path);

        // make the file_name unique
        let (filename, extension) = file_name.rsplit_once(".").unwrap();
        let file_name =
            if std::fs::metadata(format!("{}/{}", download_path, file_name)).is_ok() {
                let unique_name = Utc::now().format("%Y_%b_%d_%H_%M_%S").to_string();
                format!("{}_{}.{}", filename, unique_name, extension)
            } else {
                file_name.to_owned()
            };

        // finally open the file safely
        let file_result = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}/{}", download_path, file_name));

        (file_result.unwrap(), file_name)
    }
}
