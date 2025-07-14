// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract UniversalRouter is UniversalContract, Ownable {
    GatewayZEVM public immutable gateway;
    address public immutable uniswapRouter;
    bool public constant isUniversal = true;

    error TransferFailed();
    error InsufficientOutAmount(uint256 out, uint256 gasFee);
    error Unauthorized();
    error InvalidAddress();
    error ApproveFailed();

    event GasFeeAndOut(uint256 gasFee, uint256 out);
    event RevertEvent(string);

    event Data(bytes);

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    event MessageRelayed();

    constructor(
        address payable gatewayAddress,
        address owner,
        address uniswapRouterAddress
    ) Ownable(owner) {
        if (
            gatewayAddress == address(0) ||
            owner == address(0) ||
            uniswapRouterAddress == address(0)
        ) revert InvalidAddress();
        gateway = GatewayZEVM(gatewayAddress);
        uniswapRouter = uniswapRouterAddress;
    }

    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
        (
            bytes memory receiver,
            address targetToken,
            bytes memory data,
            uint256 gasLimit,
            RevertOptions memory revertOptions
        ) = abi.decode(
                message,
                (bytes, address, bytes, uint256, RevertOptions)
            );

        uint256 inputForGas;
        address gasZRC20;
        uint256 gasFee;
        uint256 swapAmount;

        (gasZRC20, gasFee) = IZRC20(targetToken).withdrawGasFeeWithGasLimit(
            gasLimit
        );
        if (gasZRC20 == zrc20) {
            swapAmount = amount - gasFee;
        } else {
            inputForGas = SwapHelperLib.swapTokensForExactTokens(
                uniswapRouter,
                zrc20,
                gasFee,
                gasZRC20,
                amount
            );
            swapAmount = amount - inputForGas;
        }

        uint256 outputAmount = SwapHelperLib.swapExactTokensForTokens(
            uniswapRouter,
            zrc20,
            swapAmount,
            targetToken,
            0
        );

        if (gasZRC20 == targetToken) {
            if (
                !IZRC20(gasZRC20).approve(
                    address(gateway),
                    outputAmount + gasFee
                )
            ) {
                revert ApproveFailed();
            }
        } else {
            if (!IZRC20(gasZRC20).approve(address(gateway), gasFee)) {
                revert ApproveFailed();
            }
            if (!IZRC20(targetToken).approve(address(gateway), outputAmount)) {
                revert ApproveFailed();
            }
        }

        RevertOptions memory revertOptionsUniversal = RevertOptions(
            address(this),
            true,
            revertOptions.abortAddress,
            abi.encode(
                revertOptions,
                zrc20,
                revertOptions.onRevertGasLimit,
                receiver,
                data,
                gasZRC20
            ),
            gasLimit
        );

        bytes memory msg = abi.encode(
            data,
            context.sender,
            outputAmount,
            true,
            zrc20
        );

        emit MessageRelayed();
        gateway.withdrawAndCall(
            abi.encodePacked(receiver),
            outputAmount,
            targetToken,
            msg,
            CallOptions(gasLimit, false),
            revertOptionsUniversal
        );
    }

    // onRevert is called when a contract on the destination chain reverts.
    // onRevert sends a call back to the source chain
    function onRevert(RevertContext calldata context) external onlyGateway {
        (
            RevertOptions memory revertOptions,
            address destination,
            uint256 onRevertGasLimit,
            bytes memory receiver,
            bytes memory data,
            address targetChainZRC20
        ) = abi.decode(
                context.revertMessage,
                (RevertOptions, address, uint256, bytes, bytes, address)
            );
        uint256 out = SwapHelperLib.swapExactTokensForTokens(
            uniswapRouter,
            context.asset,
            context.amount,
            destination,
            0
        );
        (, uint256 gasFee) = IZRC20(destination).withdrawGasFeeWithGasLimit(
            onRevertGasLimit
        );
        if (out < gasFee) revert InsufficientOutAmount(out, gasFee);

        if (!IZRC20(destination).approve(address(gateway), out)) {
            revert ApproveFailed();
        }
        gateway.withdrawAndCall(
            abi.encodePacked(revertOptions.revertAddress),
            out - gasFee,
            destination,
            abi.encode(data, receiver, out - gasFee, false, targetChainZRC20),
            CallOptions(onRevertGasLimit, false),
            RevertOptions(address(0), false, address(0), "", 0)
        );
    }
}
