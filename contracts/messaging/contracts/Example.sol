// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "./Messaging.sol";

contract Example is Messaging {
    event OnMessageReceiveEvent(bytes);
    event OnMessageRevertEvent();

    constructor(
        address payable gateway,
        address owner,
        address router
    ) Messaging(gateway, owner, router) {}

    function onMessageReceive(
        bytes memory data,
        address sender,
        uint256 amount
    ) internal override {
        emit OnMessageReceiveEvent(data);
    }

    function onMessageRevert(
        bytes memory data,
        address sender,
        uint256 amount
    ) internal override {
        emit OnMessageRevertEvent();
        // Revert from destination chain
    }

    function onRevert(
        RevertContext calldata context
    ) external payable override onlyGateway {
        if (context.sender != router) revert("Unauthorized");
        emit OnRevertEvent("Event from onRevert()", context);
        // Revert from ZetaChain
    }

    function sendMessage(
        address destination,
        bytes memory data,
        CallOptions memory callOptions,
        RevertOptions memory revertOptions
    ) external payable {
        bytes memory message = abi.encode(
            connected[destination],
            destination,
            data,
            callOptions,
            revertOptions
        );
        gateway.depositAndCall{value: msg.value}(
            router,
            message,
            revertOptions
        );
    }

    // Simplified function for demo purposes

    function sendSimpleMessage(
        address destination,
        string memory data
    ) external payable {
        RevertOptions memory revertOptions = RevertOptions(
            address(0),
            false,
            address(0),
            "",
            0
        );
        bytes memory message = abi.encode(
            connected[destination],
            destination,
            data,
            CallOptions(200000, false),
            revertOptions
        );
        gateway.depositAndCall{value: msg.value}(
            router,
            message,
            revertOptions
        );
    }

    // Send a message with an ERC-20 token. Currently, not supported.
    //
    // function sendMessage(
    //     address targetToken,
    //     uint256 amount,
    //     address asset,
    //     bytes memory data,
    //     CallOptions memory callOptions,
    //     RevertOptions memory revertOptions
    // ) external {
    //     bytes memory message = abi.encode(
    //         abi.encodePacked(counterparty),
    //         targetToken,
    //         data,
    //         callOptions,
    //         revertOptions
    //     );
    //     if (!IERC20(asset).transferFrom(msg.sender, address(this), amount)) {
    //         revert TransferFailed();
    //     }
    //     if (!IERC20(asset).approve(address(gateway), amount)) {
    //         revert ApprovalFailed();
    //     }

    //     gateway.depositAndCall(router, amount, asset, message, revertOptions);
    // }
}
