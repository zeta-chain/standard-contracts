// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "../shared/UniversalNFTEvents.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {ERC721URIStorageUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IWZETA.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

/**
 * @title UniversalNFTCore
 * @dev This abstract contract provides the core logic for Universal NFTs. It is designed
 *      to be imported into an OpenZeppelin-based ERC721 implementation, extending its
 *      functionality with cross-chain NFT transfer capabilities via GatewayZEVM. This
 *      contract facilitates cross-chain NFT transfers to and from ZetaChain and other
 *      connected EVM-based networks.
 */
abstract contract UniversalNFTCore is
    UniversalContract,
    ERC721Upgradeable,
    ERC721URIStorageUpgradeable,
    OwnableUpgradeable,
    UniversalNFTEvents,
    ReentrancyGuardUpgradeable
{
    // Address of the ZetaChain Gateway contract
    GatewayZEVM public gateway;

    // Address of the Uniswap Router for token swaps
    address public uniswapRouter;

    // Indicates this contract implements a Universal Contract
    bool public constant isUniversal = true;

    // Gas limit for cross-chain operations
    uint256 public gasLimitAmount;

    // Mapping of connected ZRC-20 tokens to their respective contracts
    mapping(address => bytes) public connected;

    error TransferFailed();
    error Unauthorized();
    error InvalidAddress();
    error InvalidGasLimit();
    error ApproveFailed();
    error ZeroMsgValue();
    error TokenRefundFailed();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    /**
     * @notice Sets the ZetaChain gateway contract address.
     * @dev Can only be called by the contract owner.
     * @param gatewayAddress The address of the gateway contract.
     */
    function setGateway(address gatewayAddress) external onlyOwner {
        if (gatewayAddress == address(0)) revert InvalidAddress();
        gateway = GatewayZEVM(payable(gatewayAddress));
    }

    /**
     * @notice Initializes the contract.
     * @dev Should be called during contract deployment.
     * @param gatewayAddress Address of the Gateway contract.
     * @param gasLimit Gas limit for cross-chain calls.
     * @param uniswapRouterAddress Address of the Uniswap router contract.
     */
    function __UniversalNFTCore_init(
        address gatewayAddress,
        uint256 gasLimit,
        address uniswapRouterAddress
    ) internal {
        __ReentrancyGuard_init();

        if (gatewayAddress == address(0) || uniswapRouterAddress == address(0))
            revert InvalidAddress();
        if (gasLimit == 0) revert InvalidGasLimit();
        gateway = GatewayZEVM(payable(gatewayAddress));
        uniswapRouter = uniswapRouterAddress;
        gasLimitAmount = gasLimit;
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
     * @notice Links a ZRC-20 gas token address to an NFT contract
     *         on the corresponding chain.
     * @dev Can only be called by the contract owner.
     * @param zrc20 Address of the ZRC-20 token.
     * @param contractAddress Address of the corresponding contract.
     */
    function setConnected(
        address zrc20,
        bytes calldata contractAddress
    ) external onlyOwner {
        if (zrc20 == address(0)) revert InvalidAddress();
        if (contractAddress.length == 0) revert InvalidAddress();
        connected[zrc20] = contractAddress;
        emit SetConnected(zrc20, contractAddress);
    }

    /**
     * @notice Transfers an NFT to a connected chain.
     * @dev This function accepts native ZETA tokens as gas fees, which are swapped
     *      for the corresponding ZRC-20 gas token of the destination chain. The NFT is then
     *      transferred to the destination chain using the ZetaChain Gateway.
     * @param tokenId The ID of the NFT to transfer.
     * @param receiver Address of the recipient on the destination chain.
     * @param destination Address of the ZRC-20 gas token for the destination chain.
     */
    function transferCrossChain(
        uint256 tokenId,
        address receiver,
        address destination
    ) public payable {
        _transferCrossChain(tokenId, receiver, destination);
    }

    /**
     * @notice Internal function that handles the core logic for cross-chain NFT transfer.
     * @dev This function can be overridden by child contracts to add custom functionality.
     *      It handles the gas fee calculation, token swaps, and cross-chain transfer logic.
     * @param tokenId The ID of the NFT to transfer.
     * @param receiver Address of the recipient on the destination chain.
     * @param destination Address of the ZRC-20 gas token for the destination chain.
     */
    function _transferCrossChain(
        uint256 tokenId,
        address receiver,
        address destination
    ) internal virtual nonReentrant {
        if (msg.value == 0) revert ZeroMsgValue();
        if (receiver == address(0)) revert InvalidAddress();

        string memory uri = tokenURI(tokenId);
        bytes memory message = abi.encode(
            receiver,
            tokenId,
            uri,
            0,
            msg.sender
        );

        _burn(tokenId);
        emit TokenTransfer(receiver, destination, tokenId, uri);

        (address gasZRC20, uint256 gasFee) = IZRC20(destination)
            .withdrawGasFeeWithGasLimit(gasLimitAmount);
        if (destination != gasZRC20) revert InvalidAddress();

        address WZETA = gateway.zetaToken();
        IWETH9(WZETA).deposit{value: msg.value}();
        if (!IWETH9(WZETA).approve(uniswapRouter, msg.value)) {
            revert ApproveFailed();
        }

        uint256 out = SwapHelperLib.swapTokensForExactTokens(
            uniswapRouter,
            WZETA,
            gasFee,
            gasZRC20,
            msg.value
        );

        uint256 remaining = msg.value - out;
        if (remaining > 0) {
            IWETH9(WZETA).withdraw(remaining);
            (bool success, ) = msg.sender.call{value: remaining}("");
            if (!success) revert TransferFailed();
        }

        CallOptions memory callOptions = CallOptions(gasLimitAmount, false);

        RevertOptions memory revertOptions = RevertOptions(
            address(this),
            true,
            address(this),
            abi.encode(receiver, tokenId, uri, msg.sender),
            gasLimitAmount
        );

        if (!IZRC20(gasZRC20).approve(address(gateway), gasFee)) {
            revert ApproveFailed();
        }

        gateway.call(
            connected[destination],
            destination,
            message,
            callOptions,
            revertOptions
        );
    }

    /**
     * @notice Handles cross-chain NFT transfers.
     * @dev This function is called by the Gateway contract upon receiving a message.
     *      If the destination is ZetaChain, mint an NFT and set its URI.
     *      If the destination is another chain, swap the gas token for the corresponding
     *      ZRC-20 token and use the Gateway to send a message to mint an NFT on the
     *      destination chain.
     * @param context Message context metadata.
     * @param zrc20 ZRC-20 token address.
     * @param amount Amount of token provided.
     * @param message Encoded payload containing NFT metadata.
     */
    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
        if (keccak256(context.sender) != keccak256(connected[zrc20]))
            revert Unauthorized();

        (
            address destination,
            address receiver,
            uint256 tokenId,
            string memory uri,
            address sender
        ) = abi.decode(message, (address, address, uint256, string, address));

        if (destination == address(0)) {
            _safeMint(receiver, tokenId);
            _setTokenURI(tokenId, uri);
            emit TokenTransferReceived(receiver, tokenId, uri);
        } else {
            (address gasZRC20, uint256 gasFee) = IZRC20(destination)
                .withdrawGasFeeWithGasLimit(gasLimitAmount);
            if (destination != gasZRC20) revert InvalidAddress();

            uint256 out = SwapHelperLib.swapExactTokensForTokens(
                uniswapRouter,
                zrc20,
                amount,
                destination,
                0
            );

            if (!IZRC20(destination).approve(address(gateway), out)) {
                revert ApproveFailed();
            }
            gateway.withdrawAndCall(
                connected[destination],
                out - gasFee,
                destination,
                abi.encode(receiver, tokenId, uri, out - gasFee, sender),
                CallOptions(gasLimitAmount, false),
                RevertOptions(
                    address(this),
                    true,
                    address(0),
                    abi.encode(receiver, tokenId, uri, sender),
                    0
                )
            );
        }
        emit TokenTransferToDestination(receiver, destination, tokenId, uri);
    }

    /**
     * @notice Handles a cross-chain call failure and reverts the NFT transfer.
     * @param context Metadata about the failed call.
     */
    function onRevert(RevertContext calldata context) external onlyGateway {
        (, uint256 tokenId, string memory uri, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, string, address)
        );
        _safeMint(sender, tokenId);
        _setTokenURI(tokenId, uri);
        emit TokenTransferReverted(
            sender,
            tokenId,
            uri,
            context.asset,
            context.amount
        );

        if (context.amount > 0 && context.asset != address(0)) {
            if (!IZRC20(context.asset).transfer(sender, context.amount)) {
                revert TokenRefundFailed();
            }
        }
    }

    /**
     * @notice onAbort is executed when a transfer from one connected chain to another
     * fails inside onCall, for example, because the amount of tokens supplied is not
     * sufficient to cover the withdraw gas fee to the destination and also not enough
     * to cover withdraw gas fee to the source chain. In this scenario we don't have
     * enough tokens to send NFT cross-chain, so the best thing we can do is to transfer
     * the NFT to the original sender on ZetaChain.
     * @param context Metadata about the failed call.
     */
    function onAbort(AbortContext calldata context) external onlyGateway {
        (, uint256 tokenId, string memory uri, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, string, address)
        );
        _safeMint(sender, tokenId);
        _setTokenURI(tokenId, uri);
        emit TokenTransferAborted(
            sender,
            tokenId,
            uri,
            context.outgoing,
            context.asset,
            context.amount
        );

        if (context.amount > 0 && context.asset != address(0)) {
            if (!IZRC20(context.asset).transfer(sender, context.amount)) {
                revert TokenRefundFailed();
            }
        }
    }

    /**
     * @notice Returns the metadata URI for an NFT.
     * @param tokenId The ID of the token.
     * @return The URI string.
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
     * @param interfaceId The interface ID.
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
}
