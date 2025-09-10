// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertOptions} from "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

import "../shared/UniversalTokenEvents.sol";

/**
 * @title UniversalTokenCore
 * @dev This abstract contract provides the core logic for Universal Tokens. It is designed
 *      to be imported into an OpenZeppelin-based ERC20 implementation, extending its
 *      functionality with cross-chain token transfer capabilities via GatewayEVM. This
 *      contract facilitates cross-chain token transfers to and from EVM-based networks.
 *      It's important to set the universal contract address before making cross-chain transfers.
 */
abstract contract UniversalTokenCore is
    ERC20Upgradeable,
    OwnableUpgradeable,
    UniversalTokenEvents,
    ReentrancyGuardUpgradeable
{
    // Address of the EVM gateway contract
    GatewayEVM public gateway;

    // The address of the Universal Token contract on ZetaChain. This contract serves
    // as a key component for handling all cross-chain transfers while also functioning
    // as an ERC-20 Universal Token.
    address public universal;

    // The amount of gas used when making cross-chain transfers
    uint256 public gasLimitAmount;

    error InvalidAddress();
    error Unauthorized();
    error InvalidGasLimit();
    error GasTokenTransferFailed();
    error GasTokenRefundFailed();
    error TransferToZetaChainRequiresNoGas();

    /**
     * @dev Ensures that the function can only be called by the Gateway contract.
     */
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
    function __UniversalTokenCore_init(
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
     * @notice Transfers tokens to another chain.
     * @dev Burns the tokens locally, then uses the Gateway to send a message to
     *      mint the same tokens on the destination chain. If the destination is the zero
     *      address, transfers the tokens to ZetaChain.
     * @param destination The ZRC-20 address of the gas token of the destination chain.
     * @param receiver The address on the destination chain that will receive the tokens.
     * @param amount The amount of tokens to transfer.
     */
    function transferCrossChain(
        address destination,
        address receiver,
        uint256 amount
    ) external payable {
        _transferCrossChainCommon(destination, receiver, amount, "");
    }

    /**
     * @notice Transfer tokens cross-chain and optionally call the receiver.
     * @dev Burns locally, mints on the destination, then forwards `message` to
     *      `receiver` on the destination chain.
     * @param destination ZRC-20 gas token of the destination chain; use address(0) for ZetaChain.
     * @param receiver Recipient on the destination chain.
     * @param amount Amount of tokens to transfer.
     * @param message ABI-encoded calldata executed on `receiver` after minting.
     * Payable: supply gas only when `destination != address(0)`; send 0 for ZetaChain.
     */
    function transferCrossChainAndCall(
        address destination,
        address receiver,
        uint256 amount,
        bytes memory message
    ) external payable {
        _transferCrossChainCommon(destination, receiver, amount, message);
    }

    function _transferCrossChainCommon(
        address destination,
        address receiver,
        uint256 amount,
        bytes memory extraMessage
    ) internal virtual nonReentrant {
        if (receiver == address(0)) revert InvalidAddress();

        if (destination == address(0) && msg.value > 0) {
            revert TransferToZetaChainRequiresNoGas();
        }

        bytes memory payload = abi.encode(
            destination,
            receiver,
            amount,
            msg.sender,
            extraMessage
        );

        _burn(msg.sender, amount);
        emit TokenTransfer(destination, receiver, amount);

        if (destination == address(0)) {
            gateway.call(
                universal,
                payload,
                RevertOptions(address(this), false, universal, payload, 0)
            );
        } else {
            gateway.depositAndCall{value: msg.value}(
                universal,
                payload,
                RevertOptions(
                    address(this),
                    true,
                    universal,
                    abi.encode(amount, msg.sender),
                    gasLimitAmount
                )
            );
        }
    }

    /**
     * @notice Mints tokens in response to an incoming cross-chain transfer.
     * @dev Called by the Gateway upon receiving a message.
     * @param context The message context.
     * @param message The encoded message containing information about the tokens.
     * @return A constant indicating the function was successfully handled.
     */
    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes4) {
        if (context.sender != universal) revert Unauthorized();
        (
            address receiver,
            uint256 amount,
            uint256 gasAmount,
            address sender,
            bytes memory m
        ) = abi.decode(message, (address, uint256, uint256, address, bytes));
        _mint(receiver, amount);
        if (gasAmount > 0) {
            if (sender == address(0)) revert InvalidAddress();
            (bool success, ) = payable(sender).call{value: gasAmount}("");
            if (!success) revert GasTokenTransferFailed();
        }
        if (m.length > 0) {
            (bool success, ) = receiver.call(m);
            require(success, "Call to receiver failed");
        }
        emit TokenTransferReceived(receiver, amount);
        return "";
    }

    /**
     * @notice Mints tokens and sends them back to the sender if a cross-chain transfer fails.
     * @dev Called by the Gateway if a call fails.
     * @param context The revert context containing metadata and revert message.
     */
    function onRevert(RevertContext calldata context) external onlyGateway {
        (uint256 amount, address sender) = abi.decode(
            context.revertMessage,
            (uint256, address)
        );
        _mint(sender, amount);
        if (context.amount > 0) {
            (bool success, ) = payable(sender).call{value: context.amount}("");
            if (!success) revert GasTokenRefundFailed();
        }
        emit TokenTransferReverted(
            sender,
            amount,
            address(0), // gas token
            context.amount
        );
    }

    receive() external payable {}
}
