#!/bin/bash

set -exo pipefail

yarn zetachain localnet start --force-kill --verbosity --no-analytics --exit-on-error debug &

REGISTRY_FILE="$HOME/.zetachain/localnet/registry.json"
while [ ! -f "$REGISTRY_FILE" ]; do sleep 1; done

npx hardhat compile --force --quiet

# From registry.json
ZRC20_ETHEREUM=$(jq -r '."11155112".zrc20Tokens[] | select(.symbol=="ETH.ETH") | .address' "$REGISTRY_FILE")
USDC_ETHEREUM=$(jq -r '."11155112".zrc20Tokens[] | select(.symbol=="USDC.ETH") | .originAddress' "$REGISTRY_FILE")
ZRC20_USDC_BNB=$(jq -r '."98".zrc20Tokens[] | select(.symbol=="USDC.BNB") | .address' "$REGISTRY_FILE")
ZRC20_BNB=$(jq -r '."98".zrc20Tokens[] | select(.symbol=="BNB.BNB") | .address' "$REGISTRY_FILE")
GATEWAY_ETHEREUM=$(jq -r '."11155112".contracts[] | select(.contractType=="gateway") | .address' "$REGISTRY_FILE")
GATEWAY_ZETACHAIN=$(jq -r '."31337".contracts[] | select(.contractType=="gateway") | .address' "$REGISTRY_FILE")
GATEWAY_BNB=$(jq -r '."98".contracts[] | select(.contractType=="gateway") | .address' "$REGISTRY_FILE")
UNISWAP_ROUTER=$(jq -r '."31337".contracts[] | select(.contractType=="uniswapV3Router") | .address' "$REGISTRY_FILE")
CONTRACT_REGISTRY=$(jq -r '."31337".chainInfo.registry' "$REGISTRY_FILE")
SENDER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

CONTRACT_ZETACHAIN=$(npx hardhat deploy --name UniversalRouter --network localhost --json | jq -r '.contractAddress')

CONTRACT_ETHEREUM=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_ETHEREUM" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')

CONTRACT_BNB=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_BNB" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')

npx hardhat connect --network localhost --contract "$CONTRACT_ETHEREUM" --target-chain-id 98 --target-contract "$CONTRACT_BNB" --json
npx hardhat connect --network localhost --contract "$CONTRACT_BNB"  --target-chain-id 11155112 --target-contract "$CONTRACT_ETHEREUM" --json

# Gas
npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$CONTRACT_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["string"]' alice --target-token "$ZRC20_BNB"

npx zetachain localnet check

# Source ERC-20
npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$CONTRACT_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["string"]' alice --erc20 "$USDC_ETHEREUM" --target-token "$ZRC20_BNB"

npx zetachain localnet check

# Destination ERC-20
npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$CONTRACT_BNB" --target-token "$ZRC20_USDC_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["string"]' alice

npx zetachain localnet check

# testing revert
# npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$ZRC20_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["uint256"]' 42

# yarn zetachain localnet check

yarn zetachain localnet stop