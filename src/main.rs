use clap::{App, Arg};
use reqwest::{Client, Method, header::{HeaderMap}};
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread};
use std::collections::HashMap;

use tokio;
use std::fs;
use std::io::{self, Read, BufRead};

struct SharedData {
    request_count: usize,
    response_codes: HashMap<u16, usize>,
}

async fn send_request(client: &Client, method: Method, url: &str, headers: HeaderMap, body: &str) -> reqwest::Response {
    client.request(method, url)
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to send request")
}

fn thread_function(stop_flag: Arc<AtomicBool>, shared_data: Arc<Mutex<SharedData>>, client: Client, method: Method, url: String, headers: HeaderMap, body: String) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        while !stop_flag.load(Ordering::SeqCst) {
            let response = send_request(&client, method.clone(), &url, headers.clone(), &body).await;
            
            let mut data = shared_data.lock().unwrap();
            data.request_count += 1;
            *data.response_codes.entry(response.status().as_u16()).or_insert(0) += 1;
        }
    });
}

#[tokio::main]
async fn main() {
    let matches = App::new("HTTP Requester")
        .arg(Arg::with_name("input")
            .short('i')
            .long("input")
            .value_name("FILE")
            .help("Path to the request file")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("threads")
            .short('t')
            .long("threads")
            .value_name("THREADS")
            .help("Number of threads")
            .takes_value(true)
            .default_value("4"))
        .get_matches();

    let input_path = matches.value_of("input").unwrap();
    let num_threads: usize = matches.value_of("threads").unwrap().parse().expect("Invalid number for threads");

    // Read and parse the input file (this is a simplified example, replace with actual parsing logic)
    let mut file = fs::File::open(input_path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");

    let url = "https://www.codeur.com/-seo-referencement-optimisation-trafic-hanoot"; // Replace with actual URL
    let method = Method::GET; // Replace with actual method
    let headers = HeaderMap::new(); // Replace with actual headers
    let body = ""; // Replace with actual body

    let shared_data = Arc::new(Mutex::new(SharedData {
        request_count: 0,
        response_codes: HashMap::new(),
    }));

    let stop_flag = Arc::new(AtomicBool::new(false));

    let client = Client::new();

    let mut handles = vec![];
    for _ in 0..num_threads {
        let shared_data_clone = Arc::clone(&shared_data);
        let stop_flag_clone = Arc::clone(&stop_flag);
        let client_clone = client.clone();
        let method_clone = method.clone();
        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.to_string();

        let handle = thread::spawn(move || {
            thread_function(stop_flag_clone, shared_data_clone, client_clone, method_clone, url_clone, headers_clone, body_clone);
        });
        handles.push(handle);
    }

    println!("Type 'stop' to stop the threads and exit the program.");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        if line.to_lowercase() == "stop" {
            println!("Stopping threads...");
            stop_flag.store(true, Ordering::SeqCst);
            break;
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let data = shared_data.lock().unwrap();
    println!("Total requests: {}", data.request_count);
    println!("Response codes: {:?}", data.response_codes);
}
