/* uses */

use cid::{Cid, Error as CidError};
use bao::Hash;
use blake3_processing::obao_verifier::{ObaoSlice, generate_random_chunk_index, BAO_CHUNK_SIZE};
use ipfs_api::{IpfsApi, IpfsClient, TryFromUri};
use anyhow::{Result, anyhow, Error};
use std::io::{Read, Cursor, SeekFrom};
use futures::TryStreamExt;

// TODO: Implement these using S3
// Construct a path to an obao file based on the blake3 hash.
fn gen_obao_path(obao_path: &str, hash: &Hash) -> String {
    let hash_str = &hash.to_hex();
    dbg!(hash_str);
    // Make sure you have some tests saved here!
    format!("{}/{}", obao_path, hash_str)
}

// Read the obao file from the backend.
fn read_obao(obao_path: &str) -> Result<Vec<u8>, Error> {
    dbg!(obao_path);
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
    pub obao_path: String, // The path to the obao file on the backend.
    pub client: IpfsClient, // The IPFS client to use to retrieve the file.
}

impl OracleQuery {
    // Generate a new OracleQuery.
    pub fn new(cid: Cid, hash: Hash, file_size: usize, obao_path: String, _host: String, _port: u16) -> Self {
        // Initialize our IPFS client from a specified host and port
        let client = IpfsClient::from_host_and_port(
            http::uri::Scheme::HTTP, &_host, _port
        ).unwrap();

        Self {
            cid,
            hash,
            file_size,
            obao_path,
            client
        }
    }

    // Perform the Oracle Query.
    pub async fn perform(&self) -> Result<bool, anyhow::Error> {
        /* TODO: Determine random offset and length to read from the file */
        let offset = generate_random_chunk_index(self.file_size);
        // Retrieve the file from IPFS
        match self.client
            // Read bytes of a file from IPFS
            .cat_bytes(
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
                let obao = read_obao(&gen_obao_path(&self.obao_path, &self.hash))?;
                // Create a new ObaoSlice from the retrieved file and our obao file
                let obao_slice = ObaoSlice::new(obao, &chunk, offset).unwrap();
                // Verify the file using our ObaoSlice
                Ok(obao_slice.verify(&self.hash).unwrap())

                // // Decode using our outboard encoding, and read it to the end
                // let mut decoded = Vec::new();
                // let mut decoder = bao::decode::Decoder::new_outboard(
                //     Cursor::new(&res[..]),
                //     Cursor::new(&obao[..]),
                //     &self.hash
                // );
                // // Read the decoded ObaoSlice into the decoded Vector.
                // match decoder.read_to_end(&mut decoded) {
                //     Err(e) => Ok(false),
                //     _ => Ok(true),
                // }
            } Err(e) => {
                Err(anyhow!("error getting file: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // See if cat_bytes works on a local test on a local IPFS node.
    #[tokio::test]
    async fn test_cat_bytes() {
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
            client.cat_bytes(&cid, 0, 10)
                .map_ok(|file_bytes| file_bytes.to_vec())
                .try_concat()
                .await.unwrap();

        // Assert that the bytes are the write length
        assert_eq!(bytes_res.len(), 10);

        // Assert that the bytes are the same
        assert_eq!(bytes_res, whole_res[0..10]);
    }

}