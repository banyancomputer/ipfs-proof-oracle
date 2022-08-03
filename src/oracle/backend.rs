use std::str::FromStr;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow, Error};
use s3::bucket::Bucket;
use awsregion::Region;
use awscreds::Credentials;
use std::io::Cursor;
use std::io::Read;
use std::env;
use envconfig::Envconfig;

#[macro_use]
use lazy_static::lazy_static;

// TODO: Make these configurable.
// Our S3 bucket names
const META_DATA_BUCKET: &str = "meta-data-bucket-dev-9lz7kptz8kihj7qx";
const OBAO_FILE_BUCKET: &str = "obao-file-bucket-dev-9lz7kptz8kihj7qx";
const ENDPOINT_BUCKET: &str = "endpoint-bucket-dev-9lz7kptz8kihj7qx";

lazy_static! {
    /// Our AWS region
    static ref REGION: Region = Region::UsEast2;
    // Your default AWS credentials. These should have access to the S3 buckets.
    static ref CREDENTIALS: Credentials = Credentials::new(
        Some(&dotenv::var("AWS_ACCESS_KEY_ID").unwrap()), // Access key ID
        Some(&dotenv::var("AWS_SECRET_KEY").unwrap()), // Secret access key
        None, None, None
    ).unwrap();
}

#[derive(Deserialize)]
pub struct MetaData {
    pub cid: String,
    pub hash: String,
    pub size: usize
}

// #[derive(Deserialize)]
// pub struct Endpoint {
//     pub host: String,
//     pub port: u16
// }

/// Retrieve the metadata for a file from S3.
/// # Arguments
/// * `deal_id` - The deal_id of the file to retrieve the metadata for.
/// # Returns
/// * A MetaData struct if the file exists, or raises an error if it does not.
///
/// # Example:
/// ```
/// use oracle::backend::get_meta_data;
/// let meta_data = get_meta_data("deal_id").unwrap();
/// ```
pub async fn get_meta_data(deal_id: &str) -> Result<MetaData, Error> {
    /* TODO: Get around cloning these */
    // Our AWS region
    let region = REGION.clone();
    // Our AWS credentials
    let credentials = CREDENTIALS.clone();

    // Initialize our S3 bucket
    let bucket = Bucket::new(META_DATA_BUCKET, region, credentials)?;
    // Retrieve the object from S3
    let response = bucket.get_object(deal_id).await?;
    // Read the bytes of the response into a buffer
    let mut reader = response.bytes();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    // Deserialize the buffer into a MetaData struct
    let meta_data: MetaData = serde_json::from_slice(&buffer)?;
    // Return the MetaData struct
    Ok(meta_data)
}

/// Retrieve the endpoint for a file from S3.
/// # Arguments
/// * `deal_id` - The deal_id of the file to retrieve the endpoint for.
/// # Returns
/// * A Endpoint struct if the file exists, or raises an error if it does not.
///
/// # Example
/// ```
/// use oracle::backend::get_endpoint;
/// let endpoint = get_endpoint("deal_id").unwrap();
/// ```
// pub async fn get_endpoint(deal_id: &str) -> Result<Endpoint, Error> {
//     /* TODO: Get around cloning these */
//     // Our AWS region
//     let region = REGION.clone();
//     // Our AWS credentials
//     let credentials = CREDENTIALS.clone();
//
//     // Initialize our S3 bucket
//     let bucket = Bucket::new(ENDPOINT_BUCKET, region, credentials)?;
//     // Retrieve the object from S3
//     let response = bucket.get_object(deal_id).await?;
//     // Read the bytes of the response into a buffer
//     let mut reader = response.bytes();
//     let mut buffer = Vec::new();
//     reader.read_to_end(&mut buffer)?;
//     // Deserialize the buffer into a Endpoint struct
//     let endpoint: Endpoint = serde_json::from_slice(&buffer)?;
//     // Return the Endpoint object
//     Ok(endpoint)
// }

/// Retrieve an OBAO file from S3.
/// # Arguments
/// * `cid` - The cid of the file to retrieve an obao file for.
/// # Returns
/// * A Vec<u8> containing the file if the file exists, or raises an error if it does not.
/// # Example
/// ```
/// use oracle::backend::get_obao_file;
/// let obao_file = get_obao_file("cid").unwrap();
/// ```
pub async fn get_obao_file(cid: &str) -> Result<Vec<u8>, Error> {
    /* TODO: Get around cloning these */
    // Our AWS region
    let region = REGION.clone();
    // Our AWS credentials
    let credentials = CREDENTIALS.clone();

    // Initialize our S3 bucket
    let bucket = Bucket::new(OBAO_FILE_BUCKET, region, credentials)?;
    // Retrieve the object from S3
    let response = bucket.get_object(cid).await?;
    // Read the bytes of the response into a buffer
    let mut reader = response.bytes();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    // Return the buffer
    Ok(buffer)
}