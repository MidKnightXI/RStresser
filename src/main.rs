use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let num_threads: usize = std::env::args()
        .nth(2)
        .unwrap()
        .parse::<usize>()
        .expect("Failed to parse number of threads");

    let num_requests: usize = std::env::args()
        .nth(3)
        .unwrap()
        .parse::<usize>()
        .expect("Failed to parse number of requests");

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
            let url = std::env::args().nth(1).unwrap();
            let client = Client::new();

            for _ in 0..num_requests {
                let response = client.get(url.as_str())
                    .send()
                    .await
                    .unwrap();

                // Send the response status back through the channel
                tx.send(response.status()
                    .as_u16())
                    .await
                    .unwrap();

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
