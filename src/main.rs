#![allow(dead_code, unused)]

/* uses */
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use futures::TryStreamExt;
use std::io::{self, Write};
use std::fs;
use hex::encode;
use std::str::FromStr;
use std::io::{Read, Cursor, SeekFrom};
use cid::Cid;
use std::convert::TryFrom;
use std::process::exit;
use futures::StreamExt;
use crate::oracle::{OracleQuery};

mod oracle;

// Our Request Structure for our Oracle Function
#[derive(Deserialize)]
struct Request {
    /* File meta-data */
    cid: String, // The CID of the file to be verified
    hash: String,  // The Blake3 hash of the file to be verified. Used to index the obao file.
    file_size: usize, // The size of the file to be verified. Used determine challenge blocks

    /* Retrieval meta-data */
    obao_path: String, // The path to the obao file on the backend.
    host: String, // The IPFS endpoint to use
    port: u16  // The IPFS port to use
}

// Our Response Structure from our Oracle Function
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String
}

// Our Handler Function
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let _cid_str = event.payload.cid;
    let _hash_string = event.payload.hash;
    let _file_size = event.payload.file_size;
    let obao_path = event.payload.obao_path;
    let _host = event.payload.host;
    let _port = event.payload.port;
    println!("CID: {}", &_cid_str);
    println!("Blake3 String: {}", &_hash_string);
    println!("File Size: {}", &_file_size);
    println!("Host: {}", &_host);
    println!("Port: {}", &_port);

    // Read our _cid_string into a Cid object
    let cid = Cid::try_from(_cid_str)?;
    println!("CID: {}", &cid.to_string());
    println!(" - codec: {:?}", &cid.codec());
    println!(" - version: {:?}", &cid.version());
    println!(" - hash: {:?}", &cid.hash());

    // Read the Blake3 hash in as a bao::Hash struct
    let blake3_hash = bao::Hash::from_str(&_hash_string).unwrap();
    println!("Blake3 Hash: {}", &blake3_hash.to_string());

    // Declare a variable to hold our response message
    let mut response_msg = String::new();

    // Create a new OracleQuery object
    let query = OracleQuery::new(cid, blake3_hash, _file_size, obao_path, _host, _port);
    // Perform the Oracle Query
    match query.perform().await {
        Ok(res) => {
            // If the query was successful, return a success message
            if res {
                response_msg = "Valid proof".to_string();
            } else {
                response_msg = "Failed proof".to_string();
            }
        },
        Err(err) => {
            // If the query failed, for some reason, return a failure message
            response_msg = format!("Query Failure: {}", err);
        }
    }
    println!("Response: {}", &response_msg);

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: response_msg
    };

    // Return the response
    Ok(resp)
}

// Our runtime for our Lambda Function
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
    use std::sync::Once;

    #[tokio::test]
    async fn test_handler() {
        /* Create a dummy request. Use values in tests/test_list.txt */
        let input = serde_json::from_str(
            "{\
                \"cid\": \"bafybeigiysh5xsklm4hailn25bl6ezshkzmtsewo6vbdwjvrpg7lqhz4ae\",\
                \"blake3_hash\": \"6ed8644d6b2aba69f8e21c598b8fd4d0f8fd0001d9a481508c971a2db7448303\",\
                \"file_size\": 956232,\
                \"obao_path\": \"./tests/obao\",\
                \"host\": \"127.0.0.1\",\
                \"port\": 5001\
            }"
        ).expect("failed to parse event");
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);

        let resp = function_handler(event).await.unwrap();
        assert_eq!(resp.msg, "Valid proof");
    }
}