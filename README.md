# RakNet
A fully functional RakNet implementation in rust.



#### Implementing packets using `binary_utils`

```rust
use binary_utils::*;
use rakrs::Magic;

#[derive(BinaryStream)]
pub struct SomeData {
    pub name: String,
    pub is_banned: bool
}

#[derive(BinaryStream)]
pub struct MyPacket {
    pub id: u8,
    pub magic: Magic,
    pub data: SomeData
}
```

#### Starting a RakNet Server

```rust
use rakrs::Server as RakServer;

#[tokio::main]
async fn main() {
    let mut server = RakServer::new("0.0.0.0:19132".into());
    
    // The even listener is not it's own loop
    // Events are fired as they occur
    // (they can occur in the sender or reciever thread)
    server.on(listener!(
        RakEvent::Disconnect,
        |address: String, reason: String| {
            println!("{} was disconnected because of: {}", address, reason);
        }
    ));
    
    server.on(listener!(
        RakEvent::GamePacket,
        |con: &mut Connection, buffer: &mut Vec<u8>| {
            println!("{} sent a game packet with the size of: {}",
            	con.address.to_string(),
            	buffer.len()
            );
        }
    ));
    
    server.on(listener!(
    	RakEvent::Motd,
        |address: String| -> Motd {
            Motd {
                name: "Server Name".into(),
                protocol: 420,
                player_count: 0,
                player_max: 10,
                gamemode: "Creative".into(),
                version: "1.18.0".into(),
                server_id: server.server_id.into()
            })
        }
    ));
    
    server.start().await;
}
```

