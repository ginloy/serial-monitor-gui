// mod ports;
// use std::{
//     io::{self, Write},
//     sync::{Arc, Mutex},
//     time::Duration,
// };

// use serialport::SerialPort;

// #[tokio::main]
// async fn main() {
//     let stdin = io::stdin();
//     let mut stdout = io::stdout();
//     let ports = ports::get_available_usb();
//     if ports.is_empty() {
//         println!("No usb serial ports were detected, exiting");
//         return;
//     }
//     println!("Available Ports:");
//     let mut i = 0;
//     ports.iter().for_each(|p| {
//         println!(
//             "({i})\t{}\t{}\t{}",
//             p.0,
//             p.1.manufacturer.clone().unwrap_or("unknown".to_string()),
//             p.1.product.clone().unwrap_or("unknown".to_string())
//         );
//         i += 1;
//     });

//     println!();

//     let idx = loop {
//         print!("Select Port Index: ");
//         stdout.flush().expect("IO error");
//         let mut input = String::new();
//         stdin
//             .read_line(&mut input)
//             .expect("Failed to get user input");
//         match input.trim().parse::<usize>() {
//             Ok(i) => {
//                 if i >= ports.len() {
//                     println!("Invalid port index");
//                     continue;
//                 }
//                 break i;
//             }
//             Err(_) => println!("Please enter a valid number"),
//         }
//     };

//     let bitrate = loop {
//         print!("Enter bitrate: ");
//         stdout.flush().expect("IO error");
//         let mut input = String::new();
//         stdin
//             .read_line(&mut input)
//             .expect("Failed to get user input");
//         match input.trim().parse::<u32>() {
//             Ok(rate) => break rate,
//             Err(_) => println!("Please enter a valid number"),
//         }
//     };

//     let port = ports::connect(&ports[idx].0, bitrate).unwrap();
//     let port = Arc::new(Mutex::new(port));
//     let port2 = port.clone();
//     println! {"Connected"};

//     let t1 = tokio::spawn(async move {
//         print_output(port).await;
//     });
//     let t2 = tokio::spawn(async move {
//         write_user_input(port2);
//     });
//     t1.await.unwrap();
//     t2.await.unwrap();
// }

// async fn print_output(port: Arc<Mutex<Box<dyn SerialPort>>>) {
//     let mut buf = [0; 64];
//     loop {
//         {
//             let mut port = port.lock().unwrap();
//             match port.read(&mut buf[..]) {
//                 Ok(b) => std::io::stdout().write_all(&buf[..b]).unwrap(),
//                 Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
//                 Err(e) => {
//                     eprintln!("{:?}, attempting to reconnect", e);
//                     match ports::reconnect(&port) {
//                         Ok(x) => {
//                             *port = x;
//                             println!("Connected");
//                         }
//                         Err(_) => (),
//                     }
//                 }
//             }
//         }
//         tokio::task::yield_now().await;
//     }
// }

// fn write_user_input(port: Arc<Mutex<Box<dyn SerialPort>>>) {
//     let stdin = std::io::stdin();
//     loop {
//         let mut input = String::new();
//         stdin
//             .read_line(&mut input)
//             .expect("Error reading from console");
//         let mut port = port.lock().unwrap();
//         match port.write_all(input.as_bytes()) {
//             Ok(_) => (),
//             Err(e) => {
//                 eprintln!("{:?}, attempting to reconnect", e);
//                 match ports::reconnect(&port) {
//                     Ok(x) => {
//                         *port = x;
//                         println!("Connected");
//                     }
//                     Err(_) => (),
//                 }
//             }
//         }
//     }
// }
//
//
#![allow(non_snake_case)]
mod ports;
mod ui;
mod api;

use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    dioxus_desktop::launch(ui::App);
}
