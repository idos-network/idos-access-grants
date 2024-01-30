#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
#NEAR_ENV=testnet ../../near-ts/node_modules/.bin/near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/access_grants.wasm

#NEAR_ENV=testnet ../../near-ts/node_modules/.bin/near deploy --wasmFile ./target/wasm32-unknown-unknown/release/access_grants.wasm --accountId idos-dev-2.testnet

NEAR_ENV=testnet ../../near-ts/node_modules/.bin/near deploy --wasmFile ./target/wasm32-unknown-unknown/release/access_grants.wasm --accountId idos-dev-grantees-as-public-keys.testnet
