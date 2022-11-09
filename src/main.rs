use serde_bencode::de;
use std::fs::File;
use std::io::Read;

mod torrent;

#[tokio::main]
async fn main() {
    let mut file = File::open("debian.torrent").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    println!("read {} bytes", buffer.len());

    match de::from_bytes::<torrent::Torrent>(&buffer) {
        Ok(t) => torrent::render_torrent(&t),
        Err(e) => println!("ERROR: {:?}", e),
    };
}
