// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import {CallOptions} from "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

abstract contract Messaging is Ownable {
    using SafeERC20 for IERC20;

    GatewayEVM public immutable gateway;
    address public immutable router;

    mapping(uint256 => bytes) public connected;

    event OnCallEvent(string);
    event OnRevertEvent(string, RevertContext);
    event AssetTransferred(bytes);

    error Unauthorized();
    error TransferFailed();
    error ApprovalFailed();
    error InvalidAddress();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    constructor(
        address gatewayAddress,
        address ownerAddress,
        address routerAddress
    ) Ownable(ownerAddress) {
        if (gatewayAddress == address(0) || routerAddress == address(0))
            revert InvalidAddress();
        gateway = GatewayEVM(gatewayAddress);
        router = routerAddress;
    }

    function setConnected(
        uint256 chainID,
        bytes memory contractAddress
    ) external onlyOwner {
        connected[chainID] = contractAddress;
    }

    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes memory) {
        (
            bytes memory data,
            bytes memory sender,
            uint256 amount,
            bool isCall,
            uint256 sourceChainID,
            bytes memory asset
        ) = abi.decode(message, (bytes, bytes, uint256, bool, uint256, bytes));
        if (
            context.sender != router ||
            keccak256(sender) != keccak256(connected[sourceChainID])
        ) revert Unauthorized();
        if (asset.length > 0) {
            address assetAddress = address(uint160(bytes20(asset)));
            IERC20(assetAddress).safeTransferFrom(
                msg.sender,
                address(this),
                amount
            );
        }

        if (isCall) {
            onMessageReceive(data, sender, amount, asset);
        } else {
            onMessageRevert(data, sender, amount, asset);
        }
        return "";
    }

    /// @notice Executed on the destination chain when the cross-chain call
    ///         succeeds.
    function onMessageReceive(
        bytes memory data,
        bytes memory sender,
        uint256 amount,
        bytes memory asset
    ) internal virtual;

    /// @notice Executed on the source chain when the destination execution
    ///         reverts.
    function onMessageRevert(
        bytes memory data,
        bytes memory sender,
        uint256 amount,
        bytes memory asset
    ) internal virtual;

    /// @notice Executed on the source chain when the router's onCall
    ///         reverts.
    function onRevert(RevertContext calldata context) external payable virtual;
}
