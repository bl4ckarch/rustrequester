# RustRequester

## Description
This is a Rust-based HTTP requester tool. It allows sending HTTP requests in parallel using multiple threads. The program reads configuration (like URL, method, headers, and body) from an input file and executes the requests. It tracks the number of requests made and the response codes received.

## Installation

### Prerequisites
- Rust: The project is built with Rust. If you don't have Rust installed, you can install it from [the official Rust website](https://www.rust-lang.org/tools/install).

### Setup
1. **Clone the repository:**
   ```bash
   git clone https://github.com/bl4ckarch/rustrequester.git
   cd rustrequester.git

2. **Build executable**
   ```bash
   cargo build
   cargo run -- -i path_to_input_file -t number_of_threads -r number_of_requests

- The binary is located in /target/debug/ directory

### Disclaimer

This program should not be used in an unauthorized environment,

You may not use this software for illegal or unethical purposes. This includes activities which give rise to criminal or civil liability.
Under no event shall the licensor be responsible for any activities, or misdeeds, by the licensee
