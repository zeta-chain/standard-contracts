// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../zetachain/UniversalToken.sol";

contract Universal is UniversalToken {
    constructor(
        address payable gatewayAddress,
        address owner,
        string memory name,
        string memory symbol,
        uint256 gasLimit,
        address uniswapRouter
    )
        UniversalToken(gatewayAddress, gasLimit, uniswapRouter)
        Ownable(owner)
        ERC20(name, symbol)
    {}
}
