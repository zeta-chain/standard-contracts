// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import {SystemContract} from "@zetachain/toolkit/contracts/SystemContract.sol";
import "../shared/Events.sol";

abstract contract UniversalToken is
    ERC20,
    Ownable2Step,
    UniversalContract,
    Events
{
    GatewayZEVM public immutable gateway;
    address public immutable uniswapRouter;
    uint256 private _nextTokenId;
    bool public constant isUniversal = true;
    uint256 public immutable gasLimitAmount;

    error TransferFailed();
    error Unauthorized();
    error InvalidAddress();
    error InvalidGasLimit();
    error ApproveFailed();

    mapping(address => address) public connected;

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    constructor(
        address payable gatewayAddress,
        uint256 gas,
        address uniswapRouterAddress
    ) {
        if (gatewayAddress == address(0) || uniswapRouterAddress == address(0))
            revert InvalidAddress();
        if (gas == 0) revert InvalidGasLimit();
        gateway = GatewayZEVM(gatewayAddress);
        uniswapRouter = uniswapRouterAddress;
        gasLimitAmount = gas;
    }

    function setConnected(
        address zrc20,
        address contractAddress
    ) external onlyOwner {
        connected[zrc20] = contractAddress;
        emit SetConnected(zrc20, contractAddress);
    }

    function transferCrossChain(
        address destination,
        address receiver,
        uint256 amount
    ) public {
        if (receiver == address(0)) revert InvalidAddress();
        _burn(msg.sender, amount);

        (address gasZRC20, uint256 gasFee) = IZRC20(destination)
            .withdrawGasFeeWithGasLimit(gasLimitAmount);
        if (destination != gasZRC20) revert InvalidAddress();

        if (
            !IZRC20(destination).transferFrom(msg.sender, address(this), gasFee)
        ) revert TransferFailed();
        if (!IZRC20(destination).approve(address(gateway), gasFee)) {
            revert ApproveFailed();
        }
        bytes memory message = abi.encode(receiver, amount, 0, msg.sender);

        CallOptions memory callOptions = CallOptions(gasLimitAmount, false);

        RevertOptions memory revertOptions = RevertOptions(
            address(this),
            true,
            address(0),
            abi.encode(amount, msg.sender),
            gasLimitAmount
        );

        gateway.call(
            abi.encodePacked(connected[destination]),
            destination,
            message,
            callOptions,
            revertOptions
        );
        emit TokenTransfer(destination, receiver, amount);
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
        if (context.sender != connected[zrc20]) revert Unauthorized();
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
                    abi.encode(tokenAmount, sender),
                    0
                )
            );
        }
        emit TokenTransferToDestination(destination, receiver, amount);
    }

    function onRevert(RevertContext calldata context) external onlyGateway {
        (uint256 amount, address sender) = abi.decode(
            context.revertMessage,
            (uint256, address)
        );
        _mint(sender, amount);
        emit TokenTransferReverted(sender, amount);
    }
}
