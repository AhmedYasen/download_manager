mod commands;
mod server;
mod scheduler;

use structopt::StructOpt;
use commands::Manager;

fn main() {
    // Receive cmd args
    let args = Manager::from_args();

    // execute commands
    let res = args.handle();


    // print response
    if let Err(e) = res {
        println!("Error: {:?}", e);
    } else {
        println!("{}", res.unwrap());
    }
}
