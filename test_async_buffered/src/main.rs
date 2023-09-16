use std::thread::sleep;

use futures::StreamExt;
use rand::Rng;
use tokio::{pin, task::JoinError};
use tokio_stream::{self as stream};
use tracing::{event, Level};

async fn computation(n: (u8, u8)) -> (u8, u8) {
    event!(Level::INFO, "Started Computation: {}, Delay: {}", n.1, n.0);
    sleep(std::time::Duration::from_millis(n.0.into()));
    n
}

fn create_stream() -> impl stream::Stream<Item = Result<(u8, u8), JoinError>> {
    let vec_size = 7;
    let range = 100..=255;
    let buffer_size = 3;

    let mut rng = rand::thread_rng();
    let vec: Vec<_> = (0..vec_size)
        .map(|_| rng.gen_range(range.to_owned()))
        .zip(1..)
        .collect();
    println!("{:?}", &vec);
    stream::iter(vec)
        .map(|n| tokio::spawn(computation(n)))
        // it seems that we have to use buffered to execute the tasks concurrently
        // else we will just get a JoinHandle and the tasks will be executed sequentially
        .buffered(buffer_size)
}

//removed Result
fn create_stream_r<S>(stream: S) -> impl stream::Stream<Item = (u8, u8)>
where
    S: stream::Stream<Item = Result<(u8, u8), JoinError>>,
{
    stream.filter_map(|n| async move {
        match n {
            Ok(n) => Some(n),
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        }
    })
}

//removed Result
fn create_stream_r2<S>(stream: S) -> impl stream::Stream<Item = (u8, u8)>
where
    S: stream::Stream<Item = Result<(u8, u8), JoinError>>,
{
    stream.filter_map(|s| async move { s.ok() })
}

/// Testing how to use a buffered stream
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    event!(Level::INFO, "Starting");

    let stream = create_stream();
    pin!(stream);

    event!(Level::INFO, "JoinError");
    while let Some(n) = stream.next().await {
        let n = n.unwrap();
        println!("Collected Task: {}, Delay: {}", n.1, n.0);
    }

    let stream = create_stream_r2(create_stream());
    pin!(stream);

    event!(Level::INFO, "without JoinError");
    while let Some(n) = stream.next().await {
        // let n = n.unwrap();
        println!("Collected Task: {}, Delay: {}", n.1, n.0);
    }
}
