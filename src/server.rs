use crate::conn::{Connection, ConnectionState, RecievePacketFn};
use crate::util::{from_tokenized, tokenize_addr};
use crate::Motd;
use binary_utils::*;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
// use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

pub enum RakNetVersion {
    MinecraftRecent,
    V10,
    V6,
}

impl RakNetVersion {
    pub fn to_u8(&self) -> u8 {
        match self {
            RakNetVersion::MinecraftRecent => 10,
            RakNetVersion::V10 => 10,
            RakNetVersion::V6 => 6,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RakNetEvent {
    /// When a connection is created
    ///
    /// ! This is not the same as connecting to the server !
    ///
    /// **Tuple Values**:
    /// 1. The parsed `ip:port` address of the connection.
    ConnectionCreated(String),
    /// When a connection disconnects from the server
    /// Or the server forces the connection to disconnect
    ///
    /// **Tuple Values**:
    /// 1. The parsed `ip:port` address of the connection.
    /// 2. The reason for disconnect.
    Disconnect(String, String),
    /// When a connection is sent a motd.
    ///
    /// **Tuple Values**:
    /// 1. The parsed `ip:port` address of the connection.
    MotdGeneration(String),
}

pub type RakEventListenerFn = dyn FnMut(&RakNetEvent) + Send + Sync;

pub struct RakNetServer {
    pub address: String,
    pub version: RakNetVersion,
    pub connections: Arc<Mutex<HashMap<String, Connection>>>,
    pub start_time: SystemTime,
    stop: bool,
}

impl RakNetServer {
    pub fn new(address: String) -> Self {
        Self {
            address,
            version: RakNetVersion::MinecraftRecent,
            connections: Arc::new(Mutex::new(HashMap::new())),
            start_time: SystemTime::now(),
            stop: false,
            // motd: Motd::default(),
        }
    }

    pub fn set_motd(&mut self, motd: Motd) {
        // *Arc::get_mut(&mut self.motd).unwrap() = motd;
    }

    /// Sends a stream to the specified address.
    /// Instant skips the tick and forcefully sends the packet to the client.
    pub fn send_stream(&mut self, address: String, stream: Vec<u8>, instant: bool) {
        let clients = self.connections.lock();
        match clients.unwrap().get_mut(&address) {
            Some(c) => c.send(stream, instant),
            None => return,
        };
    }


    pub async fn start(&mut self) -> io::Result<()> {
        let s = UdpSocket::bind(&self.address).await?;
        let socket = Arc::new(s);
        let sockv2 = Arc::clone(&socket);
        let (rc, st) = {
            let connections = Arc::clone(&self.connections);
            let m = Arc::clone(&connections);
            (connections, m)
        };
        let server_time = Arc::new(self.start_time);
        let should_stop = Arc::new(self.stop);
        let shouldstp = Arc::clone(&should_stop);

        let recv = tokio::spawn(async move {
            loop {
                let mut buf = [0; 2048];
                let (_len, addr) = sockv2.recv_from(&mut buf).await.expect("Failed to read address.");
                let remote = tokenize_addr(addr);

                println!("Got: {:?}", buf);

                let mut connections = rc.as_ref().lock()
                    .expect("Clients could not be safely locked.");

                if !connections.contains_key(&remote) {
                    // connection doesn't exist, make it
                    connections.insert(
                        remote.clone(),
                        Connection::new(
                            addr,
                            server_time.as_ref().clone()
                        ),
                    );
                }

                let client = match connections.get_mut(&remote) {
                    Some(c) => c,
                    None => continue,
                };

                client.recv(&buf.to_vec());

                drop(connections);

                if should_stop.as_ref() == &true {
                    break;
                }
            }
        });

        let send = tokio::spawn(async move {
            loop {
                // check clients
                sleep(Duration::from_millis(50)).await;
                let mut connections = st.as_ref().lock()
                    .expect("Could not safely mutate through connections.");

                for (addr, conn) in connections.iter_mut() {
                    conn.do_tick();

                    // do event dispatching here...

                    if conn.state == ConnectionState::Offline {
                        drop(conn);
                        continue;
                    }

                    if conn.send_queue.len() == 0 {
                        continue;
                    }

                    // get send queue
                    for pk in conn.send_queue.clone() {
                        unsafe {
                            socket.send_to(&pk[..], addr).await;
                        };
                    }

                    conn.send_queue.clear();
                }

                if shouldstp.as_ref() == &true {
                    break;
                }
            }
        });

        // this might be abuse of handles???
        (send.await, recv.await);
        Ok(())
    }

}
