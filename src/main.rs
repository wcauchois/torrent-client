use nom::bytes::complete::take_while_m_n;
use nom::combinator::{map, map_res};
use nom::error::VerboseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;
use std::iter::Iterator;
use std::net::Ipv4Addr;

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

#[derive(Debug)]
struct PeersList {
    peers: Vec<(Ipv4Addr, u16)>,
}

impl<'de> Deserialize<'de> for PeersList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf: ByteBuf = Deserialize::deserialize(deserializer)?;

        fn byte(input: &[u8]) -> IResult<&[u8], u8> {
            // It's not easy to parse a single byte: https://github.com/Geal/nom/issues/1054
            map(take_while_m_n(1, 1, |_| true), |bytes: &[u8]| bytes[0])(input)
        }

        fn ip_and_port(input: &[u8]) -> IResult<&[u8], (Ipv4Addr, u16)> {
            let (input, ip_bytes) = tuple((byte, byte, byte, byte))(input)?;
            let ip = Ipv4Addr::new(ip_bytes.0, ip_bytes.1, ip_bytes.2, ip_bytes.3);
            let (input, port_bytes) = tuple((byte, byte))(input)?;
            let port = u16::from_be_bytes([port_bytes.0, port_bytes.1]);
            Ok((input, (ip, port)))
        }

        let buf_vec = buf.to_vec();
        // Peers is a list of IP and ports.
        let peers = match many0(ip_and_port)(&buf_vec) {
            Ok((_input, peers)) => Ok(peers),
            Err(err) => Err(serde::de::Error::custom(err.to_string())),
        }?;

        Ok(PeersList { peers })
    }
}

#[derive(Deserialize, Debug)]
struct AnnounceResponse {
    interval: i32,
    peers: PeersList,
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
