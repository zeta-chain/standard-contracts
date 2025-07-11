// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertOptions} from "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import "../shared/UniversalNFTEvents.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {ERC721URIStorageUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

/**
 * @title UniversalNFTCore
 * @dev This abstract contract provides the core logic for Universal NFTs. It is designed
 *      to be imported into an OpenZeppelin-based ERC721 implementation, extending its
 *      functionality with cross-chain NFT transfer capabilities via GatewayEVM. This
 *      contract facilitates cross-chain NFT transfers to and from EVM-based networks.
 *      It's important to set the universal contract address before making cross-chain transfers.
 */
abstract contract UniversalNFTCore is
    ERC721Upgradeable,
    ERC721URIStorageUpgradeable,
    OwnableUpgradeable,
    UniversalNFTEvents,
    ReentrancyGuardUpgradeable
{
    // Address of the EVM gateway contract
    GatewayEVM public gateway;

    // The address of the Universal NFT contract on ZetaChain. This contract serves
    // as a key component for handling all cross-chain transfers while also functioning
    // as an ERC-721 Universal NFT.
    address public universal;

    // The amount of gas used when making cross-chain transfers
    uint256 public gasLimitAmount;

    error InvalidAddress();
    error Unauthorized();
    error InvalidGasLimit();
    error GasTokenTransferFailed();
    error GasTokenRefundFailed();
    error TransferToZetaChainRequiresNoGas();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    /**
     * @notice Sets the gas limit for cross-chain transfers.
     * @dev Can only be called by the contract owner.
     * @param gas New gas limit value.
     */
    function setGasLimit(uint256 gas) external onlyOwner {
        if (gas == 0) revert InvalidGasLimit();
        gasLimitAmount = gas;
    }

    /**
     * @notice Sets the universal contract address.
     * @dev Can only be called by the contract owner.
     * @param contractAddress The address of the universal contract.
     */
    function setUniversal(address contractAddress) external onlyOwner {
        if (contractAddress == address(0)) revert InvalidAddress();
        universal = contractAddress;
        emit SetUniversal(contractAddress);
    }

    /**
     * @notice Sets the EVM gateway contract address.
     * @dev Can only be called by the contract owner.
     * @param gatewayAddress The address of the gateway contract.
     */
    function setGateway(address gatewayAddress) external onlyOwner {
        if (gatewayAddress == address(0)) revert InvalidAddress();
        gateway = GatewayEVM(gatewayAddress);
    }

    /**
     * @notice Initializes the contract with gateway, universal, and gas limit settings.
     * @dev To be called during contract deployment.
     * @param gatewayAddress The address of the gateway contract.
     * @param universalAddress The address of the universal contract.
     * @param gasLimit The gas limit to set.
     */
    function __UniversalNFTCore_init(
        address gatewayAddress,
        address universalAddress,
        uint256 gasLimit
    ) internal {
        __ReentrancyGuard_init();

        if (gatewayAddress == address(0)) revert InvalidAddress();
        if (universalAddress == address(0)) revert InvalidAddress();
        if (gasLimit == 0) revert InvalidGasLimit();
        gateway = GatewayEVM(gatewayAddress);
        universal = universalAddress;
        gasLimitAmount = gasLimit;
    }

    /**
     * @notice Transfers an NFT to another chain.
     * @dev Burns the NFT locally, then uses the Gateway to send a message to
     *      mint the same NFT on the destination chain. If the destination is the zero
     *      address, transfers the NFT to ZetaChain.
     * @param tokenId The ID of the NFT to transfer.
     * @param receiver The address on the destination chain that will receive the NFT.
     * @param destination The ZRC-20 address of the gas token of the destination chain.
     */
    function transferCrossChain(
        uint256 tokenId,
        address receiver,
        address destination
    ) external payable {
        _transferCrossChain(tokenId, receiver, destination);
    }

    /**
     * @notice Internal function that handles the core logic for cross-chain NFT transfer.
     * @dev This function can be overridden by child contracts to add custom functionality.
     * @param tokenId The ID of the NFT to transfer.
     * @param receiver The address on the destination chain that will receive the NFT.
     * @param destination The ZRC-20 address of the gas token of the destination chain.
     */
    function _transferCrossChain(
        uint256 tokenId,
        address receiver,
        address destination
    ) internal virtual nonReentrant {
        if (receiver == address(0)) revert InvalidAddress();

        if (destination == address(0) && msg.value > 0) {
            revert TransferToZetaChainRequiresNoGas();
        }

        string memory uri = tokenURI(tokenId);
        bytes memory message = abi.encode(
            destination,
            receiver,
            tokenId,
            uri,
            msg.sender
        );

        _burn(tokenId);
        emit TokenTransfer(destination, receiver, tokenId, uri);

        if (destination == address(0)) {
            gateway.call(
                universal,
                message,
                RevertOptions(address(this), false, universal, message, 0)
            );
        } else {
            gateway.depositAndCall{value: msg.value}(
                universal,
                message,
                RevertOptions(
                    address(this),
                    true,
                    universal,
                    abi.encode(receiver, tokenId, uri, msg.sender),
                    gasLimitAmount
                )
            );
        }
    }

    /**
     * @notice Mint an NFT in response to an incoming cross-chain transfer.
     * @dev Called by the Gateway upon receiving a message.
     * @param context The message context.
     * @param message The encoded message containing information about the NFT.
     * @return A constant indicating the function was successfully handled.
     */
    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes4) {
        if (context.sender != universal) revert Unauthorized();

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

    /**
     * @notice Mint an NFT and send it back to the sender if a cross-chain transfer fails.
     * @dev Called by the Gateway if a call fails.
     * @param context The revert context containing metadata and revert message.
     */
    function onRevert(
        RevertContext calldata context
    ) external payable onlyGateway {
        (, uint256 tokenId, string memory uri, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, string, address)
        );
        _safeMint(sender, tokenId);
        _setTokenURI(tokenId, uri);
        if (context.amount > 0) {
            (bool success, ) = payable(sender).call{value: context.amount}("");
            if (!success) revert GasTokenRefundFailed();
        }
        emit TokenTransferReverted(
            sender,
            tokenId,
            uri,
            address(0), // gas token
            context.amount
        );
    }

    /**
     * @notice Gets the token URI for a given token ID.
     * @param tokenId The ID of the token.
     * @return The token URI as a string.
     */
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

    /**
     * @notice Checks if the contract supports a specific interface.
     * @param interfaceId The interface identifier to check.
     * @return True if the interface is supported, false otherwise.
     */
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

    receive() external payable {}
}
