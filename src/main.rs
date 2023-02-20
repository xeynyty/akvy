use std::process::exit;
use std::sync::{Arc, Mutex};

use argparse::{ArgumentParser, Store};

use hyper::{Client, Uri};

use lazy_static::lazy_static;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{Duration, Instant};
use tokio::time;

lazy_static! {
    static ref REQUESTS: Arc<Mutex<u128>> = Arc::new(Mutex::new(0));
    static ref ERRORS: Arc<Mutex<u128>> = Arc::new(Mutex::new(0));
}

#[tokio::main]
async fn main() {

    let mut url_in = String::from("http://localhost:8080");
    let mut rps: u16 = 100;

    // Args parse
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Set app parameters");
        ap.refer(&mut url_in)
            .add_option(
                &["-u", "--url"],
                Store,
                "Target URL for bench");
        ap.refer(&mut rps)
            .add_option(
                &["-r", "--rps"],
                Store,
                "Target number of requests per second"
            );
        ap.parse_args_or_exit();
    }

    let url = parse_url(url_in);
    println!("\n{} | {}", url, rps);

    let start = Instant::now();

    let mut interval = time::interval(Duration::from_micros(1_000_000 / rps as u64));

    // main worker thread
    tokio::spawn(async move {
        loop {
            let url = url.clone();

            tokio::spawn(async move {
                get(url).await;
            });

            interval.tick().await;
        }
    });


    // shutdown signal check
    let mut stream = signal(SignalKind::interrupt()).unwrap();
    loop {
        stream.recv().await;
        break;
    }

    let end = start.elapsed();

    // result block
    {
        let req = *REQUESTS.lock().unwrap();
        let err = *ERRORS.lock().unwrap();
        print!("\n\n");
        println!("Elapsed:             {:.2?}", end);
        println!("Requests:            {}", &req);
        println!("Errors:              {}", err);
        println!("Percent of errors:   {:.2}%", percent_of_errors(&req, &err))
    }

    exit(0)
}

async fn get(uri: Uri) {
    let client = Client::new();

    let resp = client.get(uri).await;

    {
        *REQUESTS.lock().unwrap() += 1;
    }

    if resp.is_err() {
        *ERRORS.lock().unwrap() += 1;
        return;
    }
    if !resp.unwrap().status().is_success() {
        *ERRORS.lock().unwrap() += 1;
    }
}

fn parse_url(url: String) -> Uri {
    if !url.contains("https://") {
        let uri = url.parse();
        if uri.is_err() {
            println!("URL error!");
            exit(1)
        }
        return uri.unwrap();
    }
    println!("App work only with HTTP!");
    exit(1)
}
fn percent_of_errors(req: &u128, err: &u128) -> f32 {
    let res = (*err as f32 / *req as f32) * 100.0;
    if res > 0 as f32 {
        return res
    } else {
        0 as f32
    }
}