use clap::{App, Arg};
use reqwest::{Client, Method, header::{HeaderMap}};
use std::{sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicBool, Ordering}}, thread};
use std::collections::HashMap;
use tokio;
use std::fs;
use std::io::{self, Read, BufRead};
use std::time::{Instant, Duration};

struct SharedData {
    request_count: AtomicUsize,
    response_codes: Mutex<HashMap<u16, usize>>,
}

async fn send_request(client: &Client, method: Method, url: &str, headers: HeaderMap, body: &str) -> reqwest::Response {
    client.request(method, url)
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to send request")
}

fn thread_function(
    stop_flag: Arc<AtomicBool>,
    shared_data: Arc<SharedData>,
    client: Client,
    method: Method,
    url: String,
    headers: HeaderMap,
    body: String,
    num_requests: u128,
) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        for _ in 0..num_requests {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            let response = send_request(&client, method.clone(), &url, headers.clone(), &body).await;
            
            shared_data.request_count.fetch_add(1, Ordering::SeqCst);
            let mut response_codes = shared_data.response_codes.lock().unwrap();
            *response_codes.entry(response.status().as_u16()).or_insert(0) += 1;
        }
    });
}

fn display_speed(shared_data: Arc<SharedData>, start: Instant, stop_flag: Arc<AtomicBool>) {
    loop {
        thread::sleep(Duration::from_secs(1)); // Update speed every second
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let elapsed_secs = start.elapsed().as_secs_f64();
        let total_requests = shared_data.request_count.load(Ordering::SeqCst);

        if elapsed_secs > 0.0 {
            let requests_per_second = total_requests as f64 / elapsed_secs;
            println!("Requests: {}, Time: {:.2} seconds, Speed: {:.2} requests/second", total_requests, elapsed_secs, requests_per_second);
        }
    }
}

#[tokio::main]
async fn main() {
    let banner = r#"
Your ASCII Art Banner Here
    "#; 
    println!("{}", banner);
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
            .default_value("25"))
        .arg(Arg::with_name("requests")
            .short('r')
            .long("requests")
            .value_name("REQUESTS")
            .help("Number of requests to make")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("url")
            .short('u')
            .long("url")
            .value_name("URL_TO_TEST")
            .help("URL to test")
            .takes_value(true)
            .required(true))
        .get_matches();
    let url = matches.value_of("url").unwrap();
    let num_requests: u128 = matches.value_of("requests").unwrap().parse().expect("Invalid number for requests");
    let input_path = matches.value_of("input").unwrap();
    let num_threads: u128 = matches.value_of("threads").unwrap().parse().expect("Invalid number for threads");
    let requests_per_thread: u128 = num_requests / num_threads;
    let mut remaining_requests = num_requests % num_threads;
    
    let mut file = fs::File::open(input_path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");

    // Here you should parse the contents variable to extract the actual URL, method, headers, and body.
    let url = url.to_string();  // Placeholder
    let method = Method::GET;  // Placeholder
    let headers = HeaderMap::new();  // Placeholder
    let body = "";  // Placeholder

    let shared_data = Arc::new(SharedData {
        request_count: AtomicUsize::new(0),
        response_codes: Mutex::new(HashMap::new()),
    });

    let stop_flag = Arc::new(AtomicBool::new(false));

    let client = Client::new();
    let start = Instant::now();
    let display_stop_flag = Arc::clone(&stop_flag);
    let display_shared_data = Arc::clone(&shared_data);
    let display_handle = thread::spawn(move || {
        display_speed(display_shared_data, start, display_stop_flag);
    });

    let mut handles = vec![];
    for _ in 0..num_threads {
        let shared_data_clone = Arc::clone(&shared_data);
        let stop_flag_clone = Arc::clone(&stop_flag);
        let client_clone = client.clone();
        let method_clone = method.clone();
        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.to_string();
        
        let thread_requests = if remaining_requests > 0 {
            remaining_requests -= 1;
            requests_per_thread + 1
        } else {
            requests_per_thread
        };

        let handle = thread::spawn(move || {
            thread_function(stop_flag_clone, shared_data_clone, client_clone, method_clone, url_clone, headers_clone, body_clone, thread_requests);
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

    display_handle.join().unwrap(); // Ensure the display thread has finished before final output

    let total_requests = shared_data.request_count.load(Ordering::SeqCst);
    let response_codes = shared_data.response_codes.lock().unwrap();
   // println!("Total requests: {}", total_requests);
    //println!("Response codes: {:?}", *response_codes);
}
