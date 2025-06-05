// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IWZETA.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";

import "../shared/UniversalTokenEvents.sol";

/**
 * @title UniversalTokenCore
 * @dev This abstract contract provides the core logic for Universal Tokens. It is designed
 *      to be imported into an OpenZeppelin-based ERC20 implementation, extending its
 *      functionality with cross-chain token transfer capabilities via GatewayZEVM. This
 *      contract facilitates cross-chain token transfers to and from ZetaChain and other
 *      connected EVM-based networks.
 */
abstract contract UniversalTokenCore is
    UniversalContract,
    ERC20Upgradeable,
    OwnableUpgradeable,
    UniversalTokenEvents,
    ReentrancyGuardUpgradeable
{
    // Indicates this contract implements a Universal Contract
    bool public constant isUniversal = true;

    // Address of the ZetaChain Gateway contract
    GatewayZEVM public gateway;

    // Address of the Uniswap Router for token swaps
    address public uniswapRouter;

    // Gas limit for cross-chain operations
    uint256 public gasLimitAmount;

    // Mapping of connected ZRC20 tokens to their respective contracts
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
    function __UniversalTokenCore_init(
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
     * @notice Links a ZRC20 gas token address to a contract on the corresponding chain.
     * @dev Can only be called by the contract owner.
     * @param zrc20 Address of the ZRC20 token.
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
     * @notice Transfers tokens to a connected chain.
     * @dev This function accepts native ZETA tokens as gas fees, which are swapped
     *      for the corresponding ZRC20 gas token of the destination chain. The tokens are then
     *      transferred to the destination chain using the ZetaChain Gateway.
     * @param destination Address of the ZRC20 gas token for the destination chain.
     * @param receiver Address of the recipient on the destination chain.
     * @param amount Amount of tokens to transfer.
     */
    function transferCrossChain(
        address destination,
        address receiver,
        uint256 amount
    ) public payable {
        _transferCrossChain(destination, receiver, amount);
    }

    /**
     * @notice Internal function that handles the core logic for cross-chain token transfer.
     * @dev This function can be overridden by child contracts to add custom functionality.
     *      It handles the gas fee calculation, token swaps, and cross-chain transfer logic.
     * @param destination Address of the ZRC20 gas token for the destination chain.
     * @param receiver Address of the recipient on the destination chain.
     * @param amount Amount of tokens to transfer.
     */
    function _transferCrossChain(
        address destination,
        address receiver,
        uint256 amount
    ) internal virtual nonReentrant {
        if (msg.value == 0) revert ZeroMsgValue();
        if (receiver == address(0)) revert InvalidAddress();

        bytes memory message = abi.encode(receiver, amount, 0, msg.sender);

        _burn(msg.sender, amount);
        emit TokenTransfer(destination, receiver, amount);

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
            abi.encode(receiver, amount, msg.sender),
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
     * @notice Handles cross-chain token transfers.
     * @dev This function is called by the Gateway contract upon receiving a message.
     *      If the destination is ZetaChain, mint tokens for the receiver.
     *      If the destination is another chain, swap the gas token for the corresponding
     *      ZRC20 token and use the Gateway to send a message to transfer tokens to the
     *      destination chain.
     * @param context Message context metadata.
     * @param zrc20 ZRC20 token address.
     * @param amount Amount of token provided.
     * @param message Encoded payload containing token transfer metadata.
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
            uint256 tokenAmount,
            address sender
        ) = abi.decode(message, (address, address, uint256, address));

        if (destination == address(0)) {
            _mint(receiver, tokenAmount);
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
                abi.encodePacked(connected[destination]),
                out - gasFee,
                destination,
                abi.encode(receiver, tokenAmount, out - gasFee, sender),
                CallOptions(gasLimitAmount, false),
                RevertOptions(
                    address(this),
                    true,
                    address(0),
                    abi.encode(receiver, tokenAmount, sender),
                    0
                )
            );
        }
        emit TokenTransferToDestination(destination, receiver, amount);
    }

    /**
     * @notice Handles a cross-chain call failure and reverts the token transfer.
     * @param context Metadata about the failed call.
     */
    function onRevert(RevertContext calldata context) external onlyGateway {
        (, uint256 amount, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, address)
        );
        _mint(sender, amount);
        emit TokenTransferReverted(
            sender,
            amount,
            context.asset,
            context.amount
        );

        if (context.amount > 0 && context.asset != address(0)) {
            if (!IZRC20(context.asset).transfer(sender, context.amount)) {
                revert TokenRefundFailed();
            }
        }
    }

    function onAbort(AbortContext calldata context) external onlyGateway {
        (, uint256 amount, address sender) = abi.decode(
            context.revertMessage,
            (address, uint256, address)
        );
        _mint(sender, amount);
        emit TokenTransferAborted(
            sender,
            amount,
            context.asset,
            context.amount
        );

        if (context.amount > 0 && context.asset != address(0)) {
            if (!IZRC20(context.asset).transfer(sender, context.amount)) {
                revert TokenRefundFailed();
            }
        }
    }
}
