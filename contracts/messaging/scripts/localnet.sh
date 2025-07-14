#!/bin/bash

set -e
set -x
set -o pipefail

yarn zetachain localnet start --force-kill --verbosity debug &

while [ ! -f "localnet.json" ]; do sleep 1; done

npx hardhat compile --force --quiet

ZRC20_ETHEREUM=$(jq -r '.addresses[] | select(.type=="ZRC-20 ETH on 11155112") | .address' localnet.json)
ZRC20_BNB=$(jq -r '.addresses[] | select(.type=="ZRC-20 BNB on 98") | .address' localnet.json)
GATEWAY_ETHEREUM=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="ethereum") | .address' localnet.json)
GATEWAY_ZETACHAIN=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="zetachain") | .address' localnet.json)
GATEWAY_BNB=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="bnb") | .address' localnet.json)
UNISWAP_ROUTER=$(jq -r '.addresses[] | select(.type=="uniswapRouterInstance" and .chain=="zetachain") | .address' localnet.json)
SENDER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

CONTRACT_ZETACHAIN=$(npx hardhat deploy --name UniversalRouter --network localhost --uniswap-router "$UNISWAP_ROUTER" --gateway "$GATEWAY_ZETACHAIN" --json | jq -r '.contractAddress')
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

yarn zetachain localnet check

npx hardhat transfer --network localhost --json --from "$CONTRACT_ETHEREUM" --to "$ZRC20_BNB" --gas-amount 1 --call-on-revert --revert-address "$CONTRACT_ETHEREUM" --revert-message "hello" --types '["uint256"]' 42

yarn zetachain localnet check

yarn zetachain localnet stop