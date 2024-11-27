// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "../shared/Events.sol";

abstract contract UniversalToken is ERC20, Ownable2Step, Events {
    GatewayEVM public immutable gateway;
    address public universal;
    uint256 public immutable gasLimitAmount;

    error InvalidAddress();
    error Unauthorized();
    error InvalidGasLimit();
    error GasTokenTransferFailed();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    function setUniversal(address contractAddress) external onlyOwner {
        if (contractAddress == address(0)) revert InvalidAddress();
        universal = contractAddress;
        emit SetUniversal(contractAddress);
    }

    constructor(address payable gatewayAddress, uint256 gas) {
        if (gatewayAddress == address(0)) revert InvalidAddress();
        gasLimitAmount = gas;
        gateway = GatewayEVM(gatewayAddress);
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function transferCrossChain(
        address destination,
        address receiver,
        uint256 amount
    ) external payable {
        if (receiver == address(0)) revert InvalidAddress();
        _burn(msg.sender, amount);

        bytes memory message = abi.encode(
            destination,
            receiver,
            amount,
            msg.sender
        );
        if (destination == address(0)) {
            gateway.call(
                universal,
                message,
                RevertOptions(address(this), false, address(0), message, 0)
            );
        } else {
            gateway.depositAndCall{value: msg.value}(
                universal,
                message,
                RevertOptions(
                    address(this),
                    true,
                    address(0),
                    abi.encode(amount, msg.sender),
                    gasLimitAmount
                )
            );
        }

        emit TokenTransfer(destination, receiver, amount);
    }

    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes4) {
        if (context.sender != universal) revert Unauthorized();
        (
            address receiver,
            uint256 amount,
            uint256 gasAmount,
            address sender
        ) = abi.decode(message, (address, uint256, uint256, address));
        _mint(receiver, amount);
        if (gasAmount > 0) {
            if (sender == address(0)) revert InvalidAddress();
            (bool success, ) = payable(sender).call{value: amount}("");
            if (!success) revert GasTokenTransferFailed();
        }
        emit TokenTransferReceived(receiver, amount);
        return "";
    }

    function onRevert(RevertContext calldata context) external onlyGateway {
        (uint256 amount, address receiver) = abi.decode(
            context.revertMessage,
            (uint256, address)
        );
        _mint(receiver, amount);
        emit TokenTransferReverted(receiver, amount);
    }

    receive() external payable {}

    fallback() external payable {}
}
