# TODO: Replace with Terraform
source .env

echo "Building docker image..."

docker run --rm \
    -v ${PWD}:/code \
    -v ${HOME}/.cargo/registry:/root/.cargo/registry \
    -v ${HOME}/.cargo/git:/root/.cargo/git \
    rustserverless/lambda-rust

echo "Deploying lambda..."

aws lambda create-function --function-name oracle-"${DEPLOYMENT_STAGE}" \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/release/bootstrap.zip \
  --runtime provided.al2 \
  --role arn:aws:iam::288251279596:role/LambdaAdminRole \
  --tracing-config Mode=Active