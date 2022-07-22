# ipfs-proof-oracle

> This is a proof oracle for our Network of IPFS providers.
> It utilizes Blake3 files based on the `bao` format in order to verify the 
> integrity of challenge blocks requested from the network.

## Testing
For local testing:

You need to have a running IPFS node. In one terminal   run:
```bash
ipfs daemon
```

Then add a file to the `tests/files` directory and run the following commands:
```bash
cd tests
./add_test.sh files/<file_name>
```

This adds the file to the IPFS node and writes meta-data to `tests/test_list.txt`.

You can use this meta-data to define a test for the Lambda function in `src/main.rs`. Look at the test `test_handler` for an example.

Then you just run the tests:
```bash
cargo test
```