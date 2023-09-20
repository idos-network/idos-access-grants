#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
#near dev-deploy --wasmFile build/access_grants.wasm --accountId

#NEAR_ENV=testnet near deploy --wasmFile build/access_grants.wasm --accountId idos-dev-1.testnet

NEAR_ENV=mainnet near deploy --wasmFile build/access_grants.wasm --accountId idos-dev-1.near
