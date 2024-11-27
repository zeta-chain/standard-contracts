// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../evm/UniversalToken.sol";

contract Connected is UniversalToken {
    constructor(
        address payable gatewayAddress,
        address owner,
        string memory name,
        string memory symbol,
        uint256 gasLimit
    )
        UniversalToken(gatewayAddress, gasLimit)
        Ownable(owner)
        ERC20(name, symbol)
    {}
}
