// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../zetachain/UniversalNFT.sol";

contract ZetaChainUniversalNFT is UniversalNFT {
    event MessageReceived(bytes message);

    function initializeZetaChainUniversalNFT(
        address initialOwner,
        string memory name,
        string memory symbol,
        address payable gatewayAddress,
        uint256 gasLimit,
        address uniswapRouterAddress
    ) external initializer {
        super.initialize(
            initialOwner,
            name,
            symbol,
            gatewayAddress,
            gasLimit,
            uniswapRouterAddress
        );
    }

    function hello(bytes memory message) external {
        emit MessageReceived(message);
    }
}
