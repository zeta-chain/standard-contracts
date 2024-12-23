// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertOptions} from "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import "../shared/UniversalNFTEvents.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {ERC721URIStorageUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";

abstract contract UniversalNFTTransferrable is
    ERC721Upgradeable,
    ERC721URIStorageUpgradeable,
    UniversalNFTEvents
{
    error InvalidAddress();
    error Unauthorized();
    error InvalidGasLimit();
    error GasTokenTransferFailed();

    modifier onlyGateway() {
        if (msg.sender != address(getGateway())) revert Unauthorized();
        _;
    }

    function getGateway() public view virtual returns (GatewayEVM);

    function getUniversal() public view virtual returns (address);

    function getGasLimitAmount() public view virtual returns (uint256);

    /**
     * @notice Transfers an NFT to another chain.
     * @dev    Burns the NFT locally, then sends an encoded message to the
     *         Gateway to recreate it on the destination chain (or revert if needed).
     * @param  tokenId The ID of the NFT to transfer.
     * @param  receiver The address on the destination chain that will receive the NFT.
     * @param  destination The contract address on the destination chain (or address(0) if same chain).
     */
    function transferCrossChain(
        uint256 tokenId,
        address receiver,
        address destination
    ) external payable virtual {
        if (receiver == address(0)) revert InvalidAddress();

        string memory uri = tokenURI(tokenId);

        _burn(tokenId);

        bytes memory message = abi.encode(
            destination,
            receiver,
            tokenId,
            uri,
            msg.sender
        );

        if (destination == address(0)) {
            getGateway().call(
                getUniversal(),
                message,
                RevertOptions(address(this), false, address(0), message, 0)
            );
        } else {
            getGateway().depositAndCall{value: msg.value}(
                getUniversal(),
                message,
                RevertOptions(
                    address(this),
                    true,
                    address(0),
                    abi.encode(receiver, tokenId, uri, msg.sender),
                    getGasLimitAmount()
                )
            );
        }

        emit TokenTransfer(destination, receiver, tokenId, uri);
    }

    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes4) {
        if (context.sender != getUniversal()) revert Unauthorized();

        (
            address receiver,
            uint256 tokenId,
            string memory uri,
            uint256 gasAmount,
            address sender
        ) = abi.decode(message, (address, uint256, string, uint256, address));

        _safeMint(receiver, tokenId);
        _setTokenURI(tokenId, uri);
        if (gasAmount > 0) {
            if (sender == address(0)) revert InvalidAddress();
            (bool success, ) = payable(sender).call{value: gasAmount}("");
            if (!success) revert GasTokenTransferFailed();
        }
        emit TokenTransferReceived(receiver, tokenId, uri);
        return "";
    }

    function onRevert(RevertContext calldata context) external onlyGateway {
        (, uint256 tokenId, string memory uri, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, string, address)
        );

        _safeMint(sender, tokenId);
        _setTokenURI(tokenId, uri);
        emit TokenTransferReverted(sender, tokenId, uri);
    }

    function tokenURI(
        uint256 tokenId
    )
        public
        view
        virtual
        override(ERC721Upgradeable, ERC721URIStorageUpgradeable)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(
        bytes4 interfaceId
    )
        public
        view
        virtual
        override(ERC721Upgradeable, ERC721URIStorageUpgradeable)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
