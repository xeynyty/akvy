use std::process::exit;
use std::sync::{Arc, Mutex};

use argparse::{ArgumentParser, Store};

use hyper::{Client, Uri};

use lazy_static::lazy_static;

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{Duration, Instant};
use tokio::time;

// lazy_static! {
    static REQ_TIME: Mutex<Vec<u32>> = Mutex::new(Vec::new());
    static ERRORS: Mutex<u128> = Mutex::new(0);
// }

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

    let end = start.elapsed();
    stream.recv().await;


    // block of result print
    {
        let req = REQ_TIME.lock().unwrap().to_vec();
        let err = *ERRORS.lock().unwrap();

        let min: u32 = match req.iter().min() {
            Some(min) => *min,
            None => 0
        };
        let max: u32 = match req.iter().max() {
            Some(max) => *max,
            None => 0
        };

        let sum = req.iter().sum::<u32>() as u128;
        let average: u32 = {
            if sum != 0 {
                (sum as u32 / req.len() as u32) as u32
            } else {
                0
            }
        };

        print!("\n\n");
        println!("Elapsed:             {:.2?}", end);
        println!("Requests:            {}", &req.len());
        println!("Errors:              {}", err);
        println!("Percent of errors:   {:.2}%", percent_of_errors(req.len(), &err));
        println!("Response time: \
                \n - Min:              {}ms \
                \n - Max:              {}ms \
                \n - Average:          {}ms", min, max, average);
    }

    exit(0)
}

async fn get(uri: Uri) {

    let start = Instant::now();

    let client = Client::new();

    let resp = client.get(uri).await;

    {
        REQ_TIME
            .lock()
            .unwrap()
            .push(start.elapsed().as_millis() as u32);
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
fn percent_of_errors(req: usize, err: &u128) -> f32 {

    let req = req as u128;

    let res = (*err as f32 / req as f32) * 100.0;
    if res > 0 as f32 {
        return res
    } else {
        0 as f32
    }
}
