// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

contract Hello {
    event MessageReceived(bytes message);

    function hello(bytes memory message) external {
        emit MessageReceived(message);
    }
}
