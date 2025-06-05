// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../evm/UniversalNFT.sol";

contract EVMUniversalNFT is UniversalNFT {
    function initializeEVMUniversalNFT(
        address initialOwner,
        string memory name,
        string memory symbol,
        address payable gatewayAddress,
        uint256 gasLimit
    ) external initializer {
        super.initialize(initialOwner, name, symbol, gatewayAddress, gasLimit);
    }
}
