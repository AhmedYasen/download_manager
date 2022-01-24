use std::net::TcpListener;
use std::sync::mpsc;

use crate::commands::Manager;
use crate::scheduler::Scheduler;

use super::utils::AppData;

use std::io::prelude::*;
use std::net::TcpStream;

// this server receives http requests using a tcp listner
// it runs on only one thread
// it handles only POST requests on the path /command
// the benefit of this server is to hold the scheduler run and talk to it a syncronousley
pub fn main_thread(max_active_downloads: u16, download_path: String) {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let scheduler = Scheduler::new(max_active_downloads as usize, download_path.clone());

    // the communication channels to the shceduler main thread
    let (thread_tx, server_rx) = mpsc::channel();
    let (server_tx, thread_rx) = mpsc::channel();

    // running the scheduler forever
    Scheduler::run(scheduler, thread_rx, thread_tx);
    let app_data = AppData {
        server_tx,
        server_rx,
        download_path,
    };

    // serve the requests
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection::<Manager, Result<Vec<String>, String>>(stream, &app_data);
    }
}

fn handle_connection<S, R>(mut stream: TcpStream, app_data: &AppData<Manager, Vec<String>>) {
    let mut buffer = [0; 2048];

    // reading the request
    stream.read(&mut buffer).unwrap();

    let mngr = String::from_utf8_lossy(&buffer).to_string();
    let mut headers = [httparse::EMPTY_HEADER; 16];

    // parse the headers
    let mut req = httparse::Request::new(&mut headers);
    let _http_req = req.parse(&buffer[..]);

    // check the method
    if req.method != Some("POST") {
        let _ = stream.write_all("HTTP/1.1 405 Method Not Allowed\r\n\r\n\r\n".as_bytes());
        return;
    }

    // if req.path == Some("/help") {

    // }

    // check the path
    if req.path != Some("/command") {
        let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n\r\n".as_bytes());
        return;
    }

    // extract the body
    let mngr = if let Some(mg) = mngr.rsplit_once("\r\n\r\n") {
        mg.1.to_string()
    } else {
        mngr
    };

    // remove the trailing redundunt characters (nulls)
    let mngr = mngr.split_once("\u{0}");

    // return bad request if there are no body
    if mngr.is_none() {
        let _ = stream.write_all("HTTP/1.1 400 Bad Request\r\n\r\n\r\n".as_bytes());
        return;
    }

    // deserilize the manager (which holds the commands)
    let mngr = serde_json::from_str::<Manager>(mngr.unwrap().0);

    if mngr.is_err() {
        let _ = stream.write_all("HTTP/1.1 400 Bad Request\r\n\r\n\r\n".as_bytes());
        return;
    }

    let mngr = mngr.unwrap();

    // send the command (manager) to the scheduler
    let _ = app_data.server_tx.send(mngr);

    // recev a message (command result)
    let rx = app_data.server_rx.recv().unwrap();

    // form the http response (a request line and body [no headers])
    let mut string = String::from("HTTP/1.1 200 Ok\r\n\r\n");

    for element in rx {
        string.push_str(element.as_str());
    }
    let _ = stream.write_all(&string.as_bytes());
}
