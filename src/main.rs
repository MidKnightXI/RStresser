use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task;
use reqwest::Client;

#[tokio::main]
async fn main() {
    // Set up the number of threads and requests to use
    let num_threads = 10;
    let num_requests = 100000;

    // Create a channel to receive responses from the threads
    let (tx, mut rx) = mpsc::channel(num_threads * num_requests);

    // Create shared variables to store the results
    let success = Arc::new(Mutex::new(0));
    let failure = Arc::new(Mutex::new(0));

    // Spawn the threads
    for _ in 0..num_threads {
        let tx = tx.clone();
        let success = success.clone();
        let failure = failure.clone();

        task::spawn(async move {
            // Create a client to handle the requests
            let client = Client::new();

            // Send the requests
            for _ in 0..num_requests {
                let response = client.get("http://localhost:8081/")
                    .send()
                    .await
                    .expect("Cannot send request");

                // Send the response status back through the channel
                tx.send(response.status()
                    .as_u16())
                    .await
                    .expect("Cannot send response status");

                // Update the results
                if response.status().is_success() {
                    *success.lock().await += 1;
                } else {
                    *failure.lock().await += 1;
                }
            }
        });
    }

    let mut counter = 0;

    while counter < num_threads * num_requests {
        let status = rx.recv().await.unwrap();
        if status == 200 {
            *success.lock().await += 1;
        } else {
            *failure.lock().await += 1;
        }
        counter += 1;
    }

    // Print the results
    println!("Success: {}", success.lock().await);
    println!("Failure: {}", failure.lock().await);
}
