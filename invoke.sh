source .env

aws lambda invoke \
  --cli-binary-format raw-in-base64-out \
  --function-name oracle-"${DEPLOYMENT_STAGE}" \
  --payload '{"command": "test"}' \
  output.json

echo "Output:"
cat output.json
printf "\nDone.\n"