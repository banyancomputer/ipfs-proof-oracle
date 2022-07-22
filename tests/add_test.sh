# Add the file with your Ipfs Daemon and extract the hash
cid=$(ipfs add "$1" -q --cid-version 1)
# Generate a blake3 hash for the file
hash=$(bao hash $1)
# Determine the size of the file
size=$(stat -c%s "$1")
# Generate an outboard encoding of the file
bao encode $1 --outboard "obao/${hash}"

# Output a json object for later reference
printf "{\n \"name\": \"%s\",\n \"size\": \"%s\" ,\n  \"hash\": \"%s\",\n  \"cid\": \"%s\"\n}\n" "$1" "$size" "$hash" "$cid" >> test_list.txt
