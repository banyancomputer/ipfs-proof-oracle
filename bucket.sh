# retrieve an object from a bucket using the aws cli
#
# Usage: bucket.sh <bucket> <object>
#

aws s3api get-object --bucket meta-data-bucket-dev-9lz7kptz8kihj7qx --key deal_id_test output.json