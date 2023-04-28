use std::{sync::Arc, env::args, error::Error, time::Duration};
use tokio::{sync::Mutex, task::{JoinHandle, spawn}};
use futures::future::join_all;
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

    let duration: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(vec![]));

    let mut handles: Vec<JoinHandle<()>> = vec![];

    println!("Number of threads: {}", num_threads);
    println!("Number of requests per threads: {}", num_requests);

    for _ in 0..num_threads
    {
        let success: Arc<Mutex<i32>> = success.clone();
        let failure: Arc<Mutex<i32>> = failure.clone();
        let duration: Arc<Mutex<Vec<Duration>>> = duration.clone();

        handles.push(
            spawn(async move {
                let url: String = get_arg(1);
                for _ in 0..num_requests
                {
                    let current = std::time::Instant::now();
                    let status: StatusCode = make_request(&url).await.unwrap();
                    duration.lock().await.push(current.elapsed());
                    if status.is_success() {
                        *success.lock().await += 1;
                    } else {
                        *failure.lock().await += 1;
                    }
                }
            }
        ));
    }

    join_all(handles).await;

    {
        let median: Duration = {
            let mut duration = duration.lock().await;
            duration.sort();
            duration[duration.len() / 2]
        };
        println!("Median: {}ms", median.as_millis());
    }


    {
        let average_time = {
            let duration = duration.lock().await;
            let mut total: Duration = Duration::new(0, 0);
            for time in duration.iter() {
                total += *time;
            }
            total / duration.len() as u32
        };
        println!("Average: {}ms", average_time.as_millis());
    }

    println!("Success: {}", success.lock().await);
    println!("Failure: {}", failure.lock().await);
    Ok(())
}
