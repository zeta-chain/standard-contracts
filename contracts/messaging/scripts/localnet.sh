#!/bin/bash

set -e
set -x
set -o pipefail

if [ "$1" = "start" ]; then npx hardhat localnet --exit-on-error & sleep 10; fi

echo -e "\n🚀 Compiling contracts..."
npx hardhat compile --force --quiet

ZRC20_ETHEREUM=$(jq -r '.addresses[] | select(.type=="ZRC-20 ETH on 5") | .address' localnet.json)
ZRC20_BNB=$(jq -r '.addresses[] | select(.type=="ZRC-20 BNB on 97") | .address' localnet.json)
GATEWAY_ETHEREUM=$(jq -r '.addresses[] | select(.type=="gatewayEVM" and .chain=="ethereum") | .address' localnet.json)
GATEWAY_BNB=$(jq -r '.addresses[] | select(.type=="gatewayEVM" and .chain=="bnb") | .address' localnet.json)
UNISWAP_ROUTER=$(jq -r '.addresses[] | select(.type=="uniswapRouterInstance" and .chain=="zetachain") | .address' localnet.json)
SENDER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

CONTRACT_ZETACHAIN=$(npx hardhat deploy --name UniversalRouter --network localhost --uniswap-router "$UNISWAP_ROUTER" --json | jq -r '.contractAddress')
echo -e "\n🚀 Deployed contract on ZetaChain: $CONTRACT_ZETACHAIN"

CONTRACT_ETHEREUM=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_ETHEREUM" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')
echo -e "🚀 Deployed contract on EVM chain: $CONTRACT_ETHEREUM"

CONTRACT_BNB=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_BNB" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')
echo -e "🚀 Deployed contract on BNB chain: $CONTRACT_BNB"

echo -e "\n📮 User Address: $SENDER"

echo -e "\n🔗 Setting counterparty contracts..."
npx hardhat connected-set-connected --network localhost --contract "$CONTRACT_ETHEREUM" --zrc20 "$ZRC20_BNB" --connected "$CONTRACT_BNB" --json
npx hardhat connected-set-connected --network localhost --contract "$CONTRACT_BNB"  --zrc20 "$ZRC20_ETHEREUM" --connected "$CONTRACT_ETHEREUM" --json

npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$ZRC20_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["string"]' alice

npx hardhat localnet-check

if [ "$1" = "start" ]; then npx hardhat localnet-stop; fi