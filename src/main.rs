#![allow(dead_code, unused)]

/* uses */

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use futures::TryStreamExt;
use ipfs_api::{IpfsApi, IpfsClient, TryFromUri};
use std::io::{self, Write};
use http::uri::Uri;
use std::fs;
use hex::encode;
use std::str::FromStr;
use std::io::Read;
use std::io::{Cursor, SeekFrom};
use cid::Cid;
use std::convert::TryFrom;
use koibumi_base32 as base32;
use bs58;

// How big File chunks are with Bao
// TODO: Subject to change, we need to coordinate with bao team.
pub(crate) const BAO_CHUNK_SIZE: usize = 1024;

// Our Request Structure for our Oracle Function
#[derive(Deserialize)]
struct Request {
    cid: String, // The CID of the file to be verified
    host: String, // The IPFS endpoint to use
    port: u16, // The IPFS port to use
    blake3_hash: String  // The Blake3 hash of the file to be verified. Used to index the obao file.
}

// Our Response Structure from our Oracle Function
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String
}
//
// fn read_cid(cid_string: &str) -> Result<Cid, Error> {
//     dbg!("Converting String: ", cid_string);
//     // If the string starts with "b" it's a base32 encoded CID.
//     // let mut cid_bytes = Vec::new();
//     if cid_string.starts_with("b") {
//         //encode the string as a base32 encoded byte array.
//         let cid_bytes = base32::encode(cid_string).to_string();
//         // dbg!("Converted to bytes: ", cid_bytes);
//         let cid = Cid::try_from(cid_bytes.as_bytes())?;
//         Ok(cid)
//     } else {
//         // Otherwise it's a base 58 encoded CID.
//         let cid_bytes = bs58::encode(cid_string).into_vec();
//         let cid = Cid::try_from(cid_bytes.as_bytes())?;
//         Ok(cid)
//     }
// }

// Our Handler Function
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let _cid_string = event.payload.cid;
    let host = event.payload.host;
    let port = event.payload.port;
    let _blake3_string = event.payload.blake3_hash;

    dbg!("CID: {}", &_cid_string);
    dbg!("Host: {}", &host);
    dbg!("Port: {}", &port);
    dbg!("Blake String: {}", &_blake3_string);

    // Read our _cid_string into a Cid object
    // TODO: How the hell is this supposed to work?
    let cid = read_cid(&_cid_string)?;

    // Determine where the obao file is located based on the blake3 hash
    // This is what I have on my machine. Change to your own path.
    let obao_path = format!("/home/alex/bao/obao/{}", _blake3_string);
    dbg!("OBAO Path: {}", &obao_path);

    // Read the Blake3 hash in as a bao::Hash struct
    let blake3_hash = bao::Hash::from_str(&_blake3_string).unwrap();

    /*
     *    TODO: Implement initializing client to custom host and port
     *     The commented out code generates an ambiguous error:
     *   Construct a Uri from our endpoint and port
     *   let uri = format!("http://{}:{}", host, port).parse::<Uri>().unwrap();
     *   dbg!("URI: {}", &uri);
     *   Create a new IPFS client
     *   let client = IpfsClient::build_with_base_uri(uri);
     */

    // Create a new IPFS client
    // This works for connecting to local IPFS node
    let client = IpfsClient::default();

    /* TODO: Implement reading chunks of the file from IPFS */

    // Declare a variable to hold our response message
    let mut response_msg = String::new();

    // Have our IPFS client request a file based on its CID
    match client
        .cat(&_cid_string) // TODO: Read from CID
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(res) => {
            // TODO: Is `res` interpreted as a stream of bytes?

            // TODO: Is this reading the file in as a stream?
            // Read our oboa in from our path
            let mut obao = fs::read(obao_path)?;

            // Decode using our outboard encoding, and read it to the end
            let mut decoded_output = Vec::new();
            let mut decoder = bao::decode::Decoder::new_outboard(
                Cursor::new(&res[..]),
                Cursor::new(&obao[..]),
                &blake3_hash
            );
            // Important: this is where the error occurs if the file is not decodable!
            match decoder.read_to_end(&mut decoded_output)? {
                0 => {
                    response_msg = format!("failed proof");
                }
                _ => {
                    response_msg = format!("valid proof");
                }
            }
        }
        Err(e) => {
            panic!("error getting file: {}", e)
        }
    }

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: response_msg
    };

    // Return the response
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

/* Our unit Tests */

#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[tokio::test]
    async fn test_handler() {
        let input = serde_json::from_str(
            "{\
                \"cid\": \"bafkreia5mw7jowpyklbi3cgdhmmsz7ltahcrc7g3obi6vn3huznf4qyuae\",\
                \"host\": \"127.0.0.1\",\
                \"port\": 5001,\
                \"blake3_hash\": \"18d02feb44d03805a6d674468c39ed75d32abf43372c14e8ef2b89a2fd56cd33\"\
            }"
        ).expect("failed to parse event");
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);

        let resp = function_handler(event).await.unwrap();
        assert_eq!(resp.msg, "valid proof");
    }
}