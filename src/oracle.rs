/* uses */

use cid::{Cid, Error as CidError};
use bao::Hash;
use blake3_processing::obao_verifier::ObaoSlice;
use ipfs_api::{IpfsApi, IpfsClient, TryFromUri};
use anyhow::{Result, anyhow, Error};
use std::io::{Read, Cursor, SeekFrom};
use futures::TryStreamExt;

// TODO: Implement these using S3
// Construct a path to an obao file based on the blake3 hash.
fn gen_obao_path(hash: &Hash) -> String {
    let hash_str = &hash.to_hex();
    dbg!(hash_str);
    // Make sure you have some tests saved here!
    format!("./obao/{}", hash_str)
}

// Read an obao file from your backend and return a Vec<u8> of the file data.
fn read_obao(obao_path: &str) -> Result<Vec<u8>, Error> {
    let mut file = std::fs::File::open(obao_path)?;
    let mut obao_bytes = Vec::new();
    file.read_to_end(&mut obao_bytes)?;
    Ok(obao_bytes)
}

// What we need to perform a single Oracle Query
pub struct OracleQuery {
    pub cid: Cid, // The CID of the file to be verified
    pub hash: Hash, // The Blake3 hash of the file to be verified.
    pub file_size: usize, // The size of the file to be verified.
    pub client: IpfsClient, // The IPFS client to use to retrieve the file.
}

impl OracleQuery {
    // Generate a new OracleQuery.
    pub fn new(cid: Cid, hash: Hash, file_size: usize, _host: String, _port: u16) -> Self {
        // Initialize our IPFS client from a specified host and port
        let client = IpfsClient::from_host_and_port(
            http::uri::Scheme::HTTP, &_host, _port
        ).unwrap();

        Self {
            cid,
            hash,
            file_size,
            client
        }
    }

    // Perform the Oracle Query.
    pub async fn perform(&self) -> Result<bool, anyhow::Error> {
        /* TODO: Determine random offset and length to read from the file */
        // Retrieve the file from IPFS
        match self.client
            // TODO: Implement reading chunks of the file from IPFS
            .cat(&self.cid.to_string())
            .map_ok(|file_bytes| file_bytes.to_vec())
            .try_concat()
            .await
        {
            Ok(res) => {
                // Read in our obao file from our backend
                let obao = read_obao(&gen_obao_path(&self.hash))?;

                /* Todo: Implement using our ObaoSlice struct */

                // Decode using our outboard encoding, and read it to the end
                let mut decoded = Vec::new();
                let mut decoder = bao::decode::Decoder::new_outboard(
                    Cursor::new(&res[..]),
                    Cursor::new(&obao[..]),
                    &self.hash
                );
                // Read the decoded ObaoSlice into the decoded Vector.
                match decoder.read_to_end(&mut decoded) {
                    Err(e) => Ok(false),
                    _ => Ok(true),
                }
            } Err(e) => {
                Err(anyhow!("error getting file: {}", e))
            }
        }
    }
}