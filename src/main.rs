use std::process::exit;
use std::sync::{Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed};

use argparse::{ArgumentParser, Store};

use hyper::{Client, Uri};
use hyper::client::HttpConnector;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{Duration, Instant};
use tokio::time;


mod utils;
use crate::utils::response::ResponseTime;


static ERRORS: AtomicUsize = AtomicUsize::new(0);
static RESPONSE: Mutex<ResponseTime> = Mutex::new(ResponseTime::new());


#[tokio::main]
async fn main() {

    let mut url_in = String::from("http://localhost:8080");
    let mut rps: u16 = 10_000;

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

    if rps == 0 {
        rps = 1;
    }

    let url = parse_url(url_in);
    println!("\n{} | {}", url, rps);

    let start = Instant::now();

    let mut interval = time::interval(Duration::from_micros(1_000_000 / rps as u64));

    let client = Client::new();

    // main worker thread
    tokio::spawn(async move {
        loop {
            let url = url.clone();
            let client = client.clone();

            tokio::spawn(async move {
                get(url, client).await;
            });

            interval.tick().await;
        }
    });

    // shutdown signal check
    let mut stream = signal(SignalKind::interrupt()).unwrap();
    stream.recv().await;
    let end = start.elapsed();

    result(end);

    exit(0)
}

async fn get(uri: Uri, client: Client<HttpConnector>) {

    let start = Instant::now();

    match client.get(uri).await {
        Ok(res) => {
            if !res.status().is_success() {
                ERRORS.fetch_add(1, Relaxed);
            }
        },
        Err(_) => {
            ERRORS.fetch_add(1, Relaxed);
        }
    }

    RESPONSE.lock().unwrap().add(start.elapsed().as_millis() as u32);
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

fn percent_of_errors(req: u32, err: &usize) -> f32 {
    let res = (*err as f32 / req as f32) * 100.0;
    if res > 0 as f32 {
        res
    } else {
        0 as f32
    }
}

fn result(end: Duration) {
    let req = RESPONSE.lock().unwrap();
    let err = ERRORS.load(Relaxed);

    print!("\n\n");
    println!("Elapsed:             {:.2?}", end);
    println!("Requests:            {}", req.get_count());
    println!("Errors:              {}", err);
    println!("Percent of errors:   {:.2}%", percent_of_errors(req.get_count(), &err));
    println!("Response time: \
                \n - Min:              {}ms \
                \n - Max:              {}ms \
                \n - Average:          {}ms", req.get_min(), req.get_max(), req.get_average());
}