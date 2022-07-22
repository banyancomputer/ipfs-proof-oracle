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
    blake3_hash: String,  // The Blake3 hash of the file to be verified. Used to index the obao file.
    file_size: usize, // The size of the file to be verified. Used determine challenge blocks

    /* Retrieval meta-data */
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
    let _blake3_string = event.payload.blake3_hash;
    let _file_size = event.payload.file_size;
    let _host = event.payload.host;
    let _port = event.payload.port;
    println!("CID: {}", &_cid_str);
    println!("Blake3 String: {}", &_blake3_string);
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
    let blake3_hash = bao::Hash::from_str(&_blake3_string).unwrap();
    println!("Blake3 Hash: {}", &blake3_hash.to_string());

    // Declare a variable to hold our response message
    let mut response_msg = String::new();

    // Create a new OracleQuery object
    let query = OracleQuery::new(cid, blake3_hash, _file_size, _host, _port);
    // Perform the Oracle Query
    match query.perform().await {
        Ok(res) => {
            // If the query was successful, return a success message
            response_msg = format!("Valid Proof: {}", res);
        }
        Err(err) => {
            // If the query failed, return a failure message
            response_msg = format!("Failure: {}", err);
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


    // #[tokio::test]
    // async fn test_handler() {
    //     let input = serde_json::from_str(
    //         "{\
    //             \"cid\": \"bafkreia5mw7jowpyklbi3cgdhmmsz7ltahcrc7g3obi6vn3huznf4qyuae\",\
    //             \"blake3_hash\": \"18d02feb44d03805a6d674468c39ed75d32abf43372c14e8ef2b89a2fd56cd33\"\
    //             \"file_length\": 236822,
    //             \"host\": \"127.0.0.1\",\
    //             \"port\": 5001,\
    //         }"
    //     ).expect("failed to parse event");
    //     let context = lambda_runtime::Context::default();
    //     let event = lambda_runtime::LambdaEvent::new(input, context);
    //
    //     let resp = function_handler(event).await.unwrap();
    //     assert_eq!(resp.msg, "valid proof");
    // }
}