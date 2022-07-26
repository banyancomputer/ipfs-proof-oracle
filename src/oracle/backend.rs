/* uses */
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow, Error};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::io::Cursor;
use std::io::Read;

// TODO: Make these configurable. Also is this how you do constants in Rust?
// Our S3 bucket names
const META_DATA_BUCKET: &str = "meta-data-bucket-dev-9lz7kptz8kihj7qx";
const OBAO_FILE_BUCKET: &str = "obao-file-bucket-dev-9lz7kptz8kihj7qx";
const ENDPOINT_BUCKET: &str = "endpoint-bucket-dev-9lz7kptz8kihj7qx";

// // Our AWS region
// const REGION: Region = "us-east-2".parse().unwrap();
//
// // Your default AWS credentials. These should have access to the S3 buckets.
// const CREDENTIALS: Credentials = Credentials::default().unwrap();

#[derive(Deserialize)]
pub struct MetaData {
    pub cid: String,
    pub hash: String,
    pub size: usize
}

#[derive(Deserialize)]
pub struct Endpoint {
    pub host: String,
    pub port: u16
}

// Retrieve the meta-data for a file from S3 based on a deal_id
pub async fn get_meta_data(deal_id: &str) -> Result<MetaData, Error> {
    // Our AWS region
    let region = "us-east-2".parse().unwrap();
    println!("Retrieving Credentials for S3");
    let credentials = Credentials::new(None, None, None, None, Some("s3-access"))?;

    println!("Initializing S3 client...");
    // Create a new S3 client
    let bucket = Bucket::new(META_DATA_BUCKET, region, credentials)?;
    // Create a new S3 GetObjectRequest
    let response = bucket.get_object(deal_id).await?;
    // Deserialize the response into a MetaData struct
    let meta_data: MetaData = serde_json::from_slice(response.bytes())?;
    // Return the MetaData object
    Ok(meta_data)
}

// Retrieve an endpoint from S3 based on a deal_id
pub async fn get_endpoint(deal_id: &str) -> Result<Endpoint, Error> {
    // Our AWS region
    let region: Region = "us-east-2".parse().unwrap();

    // Your default AWS credentials. These should have access to the S3 buckets.
    let credentials: Credentials = Credentials::default()?;
    // Create a new S3 client
    let bucket = Bucket::new(&ENDPOINT_BUCKET, region, credentials)?;
    // Create a new S3 request
    let response = bucket.get_object(deal_id).await?;
    // Deserialize the response into an Endpoint struct
    let endpoint: Endpoint = serde_json::from_slice(response.bytes())?;
    // Return the Endpoint object
    Ok(endpoint)
}

// Retrieve an obao file based on it's CID (used to index the file)
pub async fn get_obao_file(cid: &str) -> Result<Vec<u8>, Error> {
    // Our AWS region
    let region: Region = "us-east-2".parse().unwrap();

    // Your default AWS credentials. These should have access to the S3 buckets.
    let credentials: Credentials = Credentials::default()?;

    // Read the file from S3. See below for implementation details.
    let bucket = Bucket::new(&OBAO_FILE_BUCKET, region, credentials)?;
    let response = bucket.get_object(cid).await?;
    let mut cursor = Cursor::new(response.bytes());
    let mut file_bytes = Vec::new();
    cursor.read_to_end(&mut file_bytes)?;
    // Return the file bytes
    Ok(file_bytes)
}