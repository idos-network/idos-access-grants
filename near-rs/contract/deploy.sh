#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying contract"

NEAR_ENV=mainnet ../node_modules/.bin/near deploy idos-dev-4.near ./target/wasm32-unknown-unknown/release/access_grants.wasm
