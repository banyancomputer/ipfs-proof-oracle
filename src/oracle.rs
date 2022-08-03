/* uses */
use bao::Hash;
use cid::Cid;
use blake3_processing::obao::slicer::*;
use ipfs_api::{IpfsApi, IpfsClient, TryFromUri};
use anyhow::{Result, anyhow, Error};
use futures::TryStreamExt;
use std::str::FromStr;
use envconfig::Envconfig;
use crate::oracle::backend::{
    get_meta_data, MetaData,
    // get_endpoint, Endpoint,
    get_obao_file,
};

pub mod backend;


// What we need to perform a single Oracle Query
pub struct OracleQuery {
    pub cid: Cid, // The CID of the file to be verified
    pub hash: Hash, // The Blake3 hash of the file to be verified.
    pub size: usize, // The size of the file to be verified.

    pub client: IpfsClient, // The IPFS client to use to retrieve the file.
}

#[derive(Envconfig)]
struct OracleQueryConfig {
    #[envconfig(from = "ENDPOINT_HOST")]
    pub host: String,
    #[envconfig(from = "ENDPOINT_PORT")]
    pub port: u16,
}

impl OracleQuery {
    // Generate a new OracleQuery.
    pub fn new(cid: Cid, hash: Hash, size: usize) -> Self {
        let config = OracleQueryConfig::init_from_env().unwrap();
        // Initialize our IPFS client from a specified host and port
        let client = IpfsClient::from_host_and_port(
            http::uri::Scheme::HTTP, &config.host, config.port
        ).unwrap();

        Self {
            cid,
            hash,
            size,
            client
        }
    }

    // Perform the Oracle Query.
    pub async fn perform(&self) -> Result<bool, anyhow::Error> {
        // TODO - Eventually we will be reading the entire slice from on chain
        // and comparing it to the hash of the file. Most of this will be deprecated

        let offset = generate_random_chunk_index(self.size);
        // Retrieve the file from IPFS
        match self.client
            // Read the desired chunk from the file
            .cat_range(
                &self.cid.to_string(),  // using a CID as a key
                offset.try_into().unwrap(),  // offset to start reading from
                BAO_CHUNK_SIZE.try_into().unwrap()  // length to read
            )
            .map_ok(|file_bytes| file_bytes.to_vec())
            .try_concat()
            .await
        {
            Ok(chunk) => {
                /* TODO: Implement reading the obao as a stream */
                // Read in our obao file from our backend
                let obao = get_obao_file(&self.hash.to_hex()).await?;
                // Create a new ObaoSlice from the retrieved file and our obao file
                let obao_slice = ObaoSlice::new(obao, &chunk, offset).unwrap();
                // Verify the file using our ObaoSlice
                Ok(obao_slice.verify(&self.hash).unwrap())
            } Err(e) => {
                Err(anyhow!("error getting file: {}", e))
            }
        }
    }
}

// Generate a new OracleQuery from our backend based on a cid.
// Construct an Oracle Query based on a cid
// For now this is implemented by reading against an S3 backend.
// Eventually it will need to read data from our on-chain data.
pub async fn get_oracle_query(cid: &str) -> Result<OracleQuery, Error> {
    println!("Retrieving Query data from cid: {}", cid);
    // Read our meta-data from S3
    println!("Retrieving meta-data from S3");
    let meta_data: MetaData = get_meta_data(cid).await?;
    println!("CID: {}", &meta_data.cid);
    println!("Blake3 String: {}", &meta_data.hash);
    println!("File Size: {}", &meta_data.size);
    // Extract useful fields from the meta-data
    let cid = Cid::try_from(meta_data.cid)?;
    let size = meta_data.size;
    let hash = bao::Hash::from_str(&meta_data.hash)?;

    // println!("Retrieving IPFS endpoint from S3");
    // // Read the specified endpoint from S3
    // let endpoint = get_endpoint(deal_id).await?;
    // println!("Host: {}", &endpoint.host);
    // println!("Port: {}", &endpoint.port);
    // // Extract useful fields from the endpoint
    // let host = endpoint.host;
    // let port = endpoint.port;

    // Create a new OracleQuery object
    let query = OracleQuery::new(cid, hash, size);
    // Return the OracleQuery object
    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    // See if cat_bytes works on a local test on a local IPFS node.
    #[tokio::test]
    async fn test_cat_range() {
        let client = IpfsClient::from_host_and_port(
            http::uri::Scheme::HTTP, "localhost", 5001
        ).unwrap();
        let cid = "bafybeigiysh5xsklm4hailn25bl6ezshkzmtsewo6vbdwjvrpg7lqhz4ae";
        let whole_res =
            client.cat(&cid)
                .map_ok(|file_bytes| file_bytes.to_vec())
                .try_concat()
                .await.unwrap();
        let bytes_res =
            client.cat_range(&cid, 0, 10)
                .map_ok(|file_bytes| file_bytes.to_vec())
                .try_concat()
                .await.unwrap();

        // Assert that the bytes are the write length
        assert_eq!(bytes_res.len(), 10);

        // Assert that the bytes are the same
        assert_eq!(bytes_res, whole_res[0..10]);
    }

}