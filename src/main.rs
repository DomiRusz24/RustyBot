mod model;
pub mod heartbeat;
pub mod socket;

use std::{thread::sleep, time::Duration};
use std::env;

use heartbeat::Heartbeat;

use crate::socket::{SocketFactory};

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    let intents = env::var("BOT_INTENTS").unwrap_or("513".to_string());

    let intents: i64 = intents.parse().expect("Intents must be a positive integer!");

    let mut factory = SocketFactory::new_with_intents(intents);

    drop(intents);

    for arg in args {
        factory.create_socket(&arg).await;
    }

    let mut heartbeat = Heartbeat::new(factory.sockets.clone());

    loop {
        heartbeat.beat().await;
        sleep(Duration::from_secs(1));
    }
}
