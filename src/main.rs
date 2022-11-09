use serde_bencode::de;
use serde_bytes::ByteBuf;
use std::io::{self, Read};

mod torrent;

fn main() {
    let mut f = std::fs::File::open("debian.torrent").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    println!("read {} bytes", buffer.len());

    match de::from_bytes::<torrent::Torrent>(&buffer) {
        Ok(t) => torrent::render_torrent(&t),
        Err(e) => println!("ERROR: {:?}", e),
    };
}

/*
#[tokio::main]
async fn main() {
    println!("hello");
}
*/
