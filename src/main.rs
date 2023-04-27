use std::{sync::Arc, env::args, error::Error};
use tokio::sync::Mutex;
use tokio::task;
use reqwest::{Client, Response, StatusCode};

// Make a request to the server
async fn make_request(url: &String) -> Result<StatusCode, reqwest::Error>
{
    let client: Client = Client::new();
    let response: Response = client.get(url).send().await?;
    Ok(response.status())
}

fn get_arg(index: usize) -> String
{
    match args().nth(index)
    {
        Some(v) => return v,
        None => {
            eprintln!("Not enough argument specified");
            std::process::exit(1);
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let num_threads: i32 = get_arg(3).parse::<i32>()?;
    let num_requests: i32 = get_arg(2).parse::<i32>()? / num_threads;
    let success: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let failure: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

    println!("Number of threads: {}", num_threads);
    println!("Number of requests per threads: {}", num_requests);

    for _ in 0..num_threads
    {
        let success: Arc<Mutex<i32>> = success.clone();
        let failure: Arc<Mutex<i32>> = failure.clone();

        task::spawn(async move {
            let url: String = get_arg(1);
            for _ in 0..num_requests
            {
                let status: StatusCode = make_request(&url).await.unwrap();
                if status.is_success()
                {
                    *success.lock().await += 1;
                }
                else
                {
                    *failure.lock().await += 1;
                }
            }
        }).await?;
    }

    println!("Success: {}", success.lock().await);
    println!("Failure: {}", failure.lock().await);
    Ok(())
}
