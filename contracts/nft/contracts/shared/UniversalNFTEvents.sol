// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

contract UniversalNFTEvents {
    event SetUniversal(address indexed universalAddress);
    event SetConnected(address indexed zrc20, bytes contractAddress);
    event TokenMinted(address indexed to, uint256 indexed tokenId, string uri);
    event TokenTransfer(
        address indexed destination,
        address indexed receiver,
        uint256 indexed tokenId,
        string uri
    );
    event TokenTransferReceived(
        address indexed receiver,
        uint256 indexed tokenId,
        string uri
    );
    event TokenTransferReverted(
        address indexed sender,
        uint256 indexed tokenId,
        string uri,
        address refundAsset,
        uint256 refundAmount
    );
    event TokenTransferAborted(
        address indexed sender,
        uint256 indexed tokenId,
        string uri,
        bool outgoing,
        address refundAsset,
        uint256 refundAmount
    );
    event TokenTransferToDestination(
        address indexed destination,
        address indexed sender,
        uint256 indexed tokenId,
        string uri
    );
}
