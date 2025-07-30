#!/bin/bash

set -e
set -x
set -o pipefail

npx zetachain localnet start --force-kill --verbosity debug &

while [ ! -f "localnet.json" ]; do sleep 1; done

npx hardhat compile --force --quiet

ZRC20_ETHEREUM=$(jq -r '.addresses[] | select(.type=="ZRC-20 ETH.ETH on 11155112") | .address' localnet.json)
USDC_ETHEREUM=$(jq -r '.addresses[] | select(.type=="ERC-20 USDC.ETH" and .chain=="ethereum") | .address' localnet.json)
ZRC20_USDC_BNB=$(jq -r '.addresses[] | select(.type=="ZRC-20 USDC.BNB on 98") | .address' localnet.json)
ZRC20_BNB=$(jq -r '.addresses[] | select(.type=="ZRC-20 BNB.BNB on 98") | .address' localnet.json)
GATEWAY_ETHEREUM=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="ethereum") | .address' localnet.json)
GATEWAY_ZETACHAIN=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="zetachain") | .address' localnet.json)
GATEWAY_BNB=$(jq -r '.addresses[] | select(.type=="gateway" and .chain=="bnb") | .address' localnet.json)
UNISWAP_ROUTER=$(jq -r '.addresses[] | select(.type=="uniswapRouterInstance" and .chain=="zetachain") | .address' localnet.json)
CONTRACT_REGISTRY=$(jq -r '.addresses[] | select(.type=="coreRegistry" and .chain=="zetachain") | .address' localnet.json)
SENDER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

CONTRACT_ZETACHAIN=$(npx hardhat deploy --name UniversalRouter --network localhost --uniswap-router "$UNISWAP_ROUTER" --gateway "$GATEWAY_ZETACHAIN" --contract-registry "$CONTRACT_REGISTRY" --json | jq -r '.contractAddress')
echo -e "\n🚀 Deployed contract on ZetaChain: $CONTRACT_ZETACHAIN"

CONTRACT_ETHEREUM=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_ETHEREUM" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')
echo -e "🚀 Deployed contract on EVM chain: $CONTRACT_ETHEREUM"

CONTRACT_BNB=$(npx hardhat deploy --name Example --json --network localhost --gateway "$GATEWAY_BNB" --router "$CONTRACT_ZETACHAIN" | jq -r '.contractAddress')
echo -e "🚀 Deployed contract on BNB chain: $CONTRACT_BNB"

echo -e "\n📮 User Address: $SENDER"

echo -e "\n🔗 Setting counterparty contracts..."
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