// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../zetachain/UniversalToken.sol";

contract ZetaChainUniversalToken is UniversalToken {
    function initializeZetaChainUniversalToken(
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
}
