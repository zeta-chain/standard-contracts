// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import {CallOptions} from "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";

contract Messaging is Ownable {
    GatewayEVM public immutable gateway;
    address public router;

    event HelloEvent(string, string);
    event OnCallEvent(string);
    event OnRevertEvent(string, RevertContext);

    error Unauthorized();
    error TransferFailed();
    error ApprovalFailed();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    mapping(address => bytes) public connected;

    function setConnected(
        address zrc20,
        bytes memory contractAddress
    ) external onlyOwner {
        connected[zrc20] = contractAddress;
    }

    constructor(
        address payable gatewayAddress,
        address ownerAddress,
        address routerAddress
    ) Ownable(ownerAddress) {
        gateway = GatewayEVM(gatewayAddress);
        router = routerAddress;
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
            address zrc20
        ) = abi.decode(message, (bytes, bytes, uint256, bool, address));
        if (
            context.sender != router ||
            keccak256(sender) != keccak256(connected[zrc20])
        ) revert Unauthorized();
        if (isCall) {
            onMessageReceive(data, sender, amount);
        } else {
            onMessageRevert(data, sender, amount);
        }
        return "";
    }

    // onRevert is executed when router's onCall reverts
    function onRevert(
        RevertContext calldata context
    ) external payable virtual onlyGateway {
        if (context.sender != router) revert("Unauthorized");
        emit OnRevertEvent("Event from onRevert()", context);
    }

    function onMessageReceive(
        bytes memory data,
        bytes memory sender,
        uint256 amount
    ) internal virtual {
        // To be overridden in the child contract
    }

    function onMessageRevert(
        bytes memory data,
        bytes memory sender,
        uint256 amount
    ) internal virtual {
        // To be overridden in the child contract
    }

    receive() external payable {}
}
