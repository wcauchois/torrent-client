use serde::{Deserialize, Serialize, Serializer};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;

mod torrent;

#[derive(Serialize)]
struct AnnounceParamsExceptInfoHash {
    // info_hash: Vec<u8>,
    peer_id: String,
    port: i32,
    uploaded: i32,
    downloaded: i32,
    compact: i32,
    left: i32,
}

#[derive(Deserialize, Debug)]
struct AnnounceResponse {
    interval: i32,
    peers: ByteBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut file = File::open("debian.torrent").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    println!("read {} bytes", buffer.len());

    let t = serde_bencode::de::from_bytes::<torrent::Torrent>(&buffer)?;
    torrent::render_torrent(&t);

    let bencoded_info = serde_bencode::ser::to_bytes(&t.info)?;

    let mut hasher = Sha1::new();
    hasher.update(bencoded_info);
    let hash_result = hasher.finalize();

    let client = reqwest::Client::new();
    let announce_url = t.announce.unwrap();

    let res = client
        .get(format!(
            "{}?info_hash={}",
            announce_url,
            urlencoding::encode_binary(&hash_result)
        ))
        .query(&AnnounceParamsExceptInfoHash {
            peer_id: "-TR2940-k8hj0wgej6ch".to_string(),
            port: 1234,
            uploaded: 0,
            downloaded: 0,
            compact: 1,
            left: 100,
        })
        .send()
        .await?;

    let res_bytes = res.bytes().await?;

    let parsed_res = serde_bencode::de::from_bytes::<AnnounceResponse>(&res_bytes)?;
    println!("response: {:?}", parsed_res);

    Ok(())
}
