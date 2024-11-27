// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../zetachain/UniversalNFT.sol";

contract Universal is UniversalNFT {
    constructor(
        address payable gatewayAddress,
        address owner,
        string memory name,
        string memory symbol,
        uint256 gasLimit,
        address uniswapRouter
    )
        UniversalNFT(gatewayAddress, gasLimit, uniswapRouter)
        Ownable(owner)
        ERC721(name, symbol)
    {}
}
