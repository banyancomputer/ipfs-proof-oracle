use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

// Our Request Structure
#[derive(Deserialize)]
struct Request {
    command: String,
}

// Our Response Structure
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

// Our Handler Function
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let command = event.payload.command;

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {}.", command),
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
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
        let input = serde_json::from_str("{\"command\": \"test\"}").expect("failed to parse event");
        let context = lambda_runtime::Context::default();
        let event = lambda_runtime::LambdaEvent::new(input, context);

        let resp = function_handler(event).await.unwrap();
        assert_eq!(resp.msg, "Command test.");
    }
}