# ipfs-proof-oracle

> This is a proof oracle for our Network of IPFS providers.
> It utilizes Blake3 files based on the `bao` format in order to verify the 
> integrity of challenge blocks requested from the network.

## Testing
The Oracle reads meta-data from S3 in order to verify the integrity of a deal stored under a `dealID`.
Therefore, any meta-data that a local test requires must be stored in S3. See the repository `oracle_storage` for more information.
Make sure you have appropriately configured AWS keys in a `.env` file in the root of the repository.

Once the meta-data is stored in S3, the Oracle can be tested locally:

You need to have a running IPFS node. In one terminal run:
```bash
ipfs daemon
```

Make sure that the file you want to test is stored in the IPFS node. In another terminal run:
```
ipfs add <file_name> --cid-version 1.
```
where `<file_name>` is the name of the file you processed with `oralce_storage`.

This adds the file to the IPFS node.

You can use the file's CID to define a test for the Lambda function in `src/main.rs`. Look at the test `test_handler` for an example.
Meta-Data is indexed by the 'cid' of the file.

When this is done, you can run the test:
```bash
cargo test
```