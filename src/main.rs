use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use futures::TryStreamExt;
use std::io::{self, Write};
use std::fs;
use hex::encode;
use std::io::{Read, Cursor, SeekFrom};
use std::convert::TryFrom;
use std::process::exit;
use futures::StreamExt;
use crate::oracle::OracleQuery;

mod oracle;

// Our Request Structure for our Oracle Function
#[derive(Deserialize)]
struct Request {
    /* File meta-data */
    cid: String, // The cid of the file to be verified.
}

// Our Response Structure from our Oracle Function
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String
}

// Our Handler Function
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Get the deal_id from the request
    let _cid = event.payload.cid;

    // Construct an Oracle Query from our backend based on the deal_id
    let query: OracleQuery  = oracle::get_oracle_query(&_cid).await?;

    // Declare a variable to hold our response message
    let mut response_msg = String::new();
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
                \"cid\": \"bafybeigiysh5xsklm4hailn25bl6ezshkzmtsewo6vbdwjvrpg7lqhz4ae\"\
            }"
        ).expect("failed to parse event");
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);

        let resp = function_handler(event).await.unwrap();
        assert_eq!(resp.msg, "Valid proof");
    }
}