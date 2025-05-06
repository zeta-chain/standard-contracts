// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

contract UniversalTokenEvents {
    event SetUniversal(address indexed universalAddress);
    event SetConnected(address indexed zrc20, bytes contractAddress);
    event TokenMinted(address indexed to, uint256 amount);
    event TokenTransfer(
        address indexed destination,
        address indexed receiver,
        uint256 amount
    );
    event TokenTransferReceived(address indexed receiver, uint256 amount);
    event TokenTransferReverted(
        address indexed sender,
        uint256 amount,
        address refundAsset,
        uint256 refundAmount
    );
    event TokenTransferAborted(
        address indexed sender,
        uint256 amount,
        address refundAsset,
        uint256 refundAmount
    );
    event TokenTransferToDestination(
        address indexed destination,
        address indexed sender,
        uint256 amount
    );
}
