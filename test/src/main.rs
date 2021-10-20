use rakrs;

use rakrs::RakNetServer;
use rakrs::conn::{Connection};
use rakrs::Motd;
use rakrs::RakNetEvent;
use binary_utils::*;
use std::sync::{Arc};

#[tokio::main]
async fn main() {
     let mut server = RakNetServer::new(String::from("0.0.0.0:19132"));
     server.set_motd(Motd {
          name: "Sus!!!".to_owned(),
          protocol: 190,
          player_count: 0,
          player_max: 10000,
          gamemode: "creative".to_owned(),
          version: "1.18.9".to_owned(),
          server_id: 2747994720109207718 as i64
     });
     server.start().await;
     dbg!("Hi I am running concurrently.");
}