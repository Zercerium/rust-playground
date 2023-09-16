use std::thread::sleep;

use futures::StreamExt;
use tokio::pin;
use tokio_stream::{self as stream};
use tracing::{event, Level};

struct DummyData {
    id: u8,
    delay_1: u16,
    delay_2: u16,
}

impl DummyData {
    fn new(id: u8, delay_1: u16, delay_2: u16) -> Self {
        Self {
            id,
            delay_1,
            delay_2,
        }
    }

    fn create() -> Vec<Self> {
        vec![
            DummyData::new(1, 250, 2000),
            DummyData::new(2, 250, 2000),
            DummyData::new(3, 250, 2000),
            DummyData::new(4, 2000, 4000),
            DummyData::new(5, 2500, 200),
            DummyData::new(6, 3000, 200),
            DummyData::new(7, 1000, 200),
            DummyData::new(8, 1500, 200),
            DummyData::new(9, 2000, 200),
            DummyData::new(10, 1500, 200),
        ]
    }
}

async fn computation1(n: DummyData) -> DummyData {
    event!(
        Level::INFO,
        "Started Download: {}, Delay: {}",
        n.id,
        n.delay_1
    );
    sleep(std::time::Duration::from_millis(n.delay_1.into()));
    event!(
        Level::INFO,
        "Finished Download: {}, Delay: {}",
        n.id,
        n.delay_1
    );

    n
}

async fn computation2(n: DummyData) -> DummyData {
    event!(
        Level::INFO,
        "Started Decompress: {}, Delay: {}",
        n.id,
        n.delay_2
    );
    sleep(std::time::Duration::from_millis(n.delay_2.into()));
    event!(
        Level::INFO,
        "Finished Decompress: {}, Delay: {}",
        n.id,
        n.delay_2
    );
    n
}

/// Testing how to use a buffered stream
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    event!(Level::INFO, "Starting");
    let dummy_data = DummyData::create();

    let stream = stream::iter(dummy_data)
        .map(|n| tokio::spawn(computation1(n)))
        .buffered(1)
        .filter_map(|n| async move { n.ok() })
        .map(|n| tokio::spawn(computation2(n)))
        .buffered(1)
        .filter_map(|n| async move { n.ok() });
    pin!(stream);

    while let Some(n) = stream.next().await {
        // let n = n.unwrap();
        println!("Collected Task: {}", n.id);
    }
}
