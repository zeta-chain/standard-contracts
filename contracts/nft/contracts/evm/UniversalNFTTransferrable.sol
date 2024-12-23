// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertOptions} from "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import "../shared/UniversalNFTEvents.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";

abstract contract UniversalNFTTransferrable is
    ERC721Upgradeable,
    UniversalNFTEvents
{
    error InvalidAddress();

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
}
