// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract UniversalRouter is UniversalContract, Ownable {
    address public immutable uniswapRouter;
    bool public constant isUniversal = true;
    address public immutable contractRegistry;

    error TransferFailed();
    error InsufficientOutAmount(uint256 out, uint256 gasFee);
    error InvalidAddress();
    error ApproveFailed();
    event OnRevertEvent(
        address asset,
        uint256 amount,
        address gasZRC20,
        uint256 gasFee
    );

    event GasFeeAndOut(uint256 gasFee, uint256 out);
    event RevertEvent(string);

    event MessageRelayed();

    struct CallParams {
        bytes receiver;
        address targetToken;
        bytes data;
        uint256 gasLimit;
        RevertOptions revertOptions;
    }

    constructor(address owner) Ownable(owner) {
        if (owner == address(0)) revert InvalidAddress();
        (bool active, bytes memory uniswapRouterBytes) = registry
            .getContractInfo(block.chainid, "uniswapV2Router02");
        if (!active) revert InvalidAddress();
        uniswapRouter = address(uint160(bytes20(uniswapRouterBytes)));
    }

    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
        CallParams memory callParams = _decode(message);

        uint256 inputForGas;
        uint256 swapAmount;
        address gasZRC20;
        uint256 gasFee;

        (gasZRC20, gasFee) = IZRC20(callParams.targetToken)
            .withdrawGasFeeWithGasLimit(callParams.gasLimit);
        if (gasZRC20 == callParams.targetToken) {
            swapAmount = amount;
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
            callParams.targetToken,
            0
        );

        if (!IZRC20(gasZRC20).approve(address(gateway), gasFee)) {
            revert ApproveFailed();
        }
        if (
            !IZRC20(callParams.targetToken).approve(
                address(gateway),
                outputAmount
            )
        ) {
            revert ApproveFailed();
        }

        bytes memory asset = "";
        if (gasZRC20 != callParams.targetToken) {
            (, , , asset, , ) = IBaseRegistry(registry).getZRC20TokenInfo(
                callParams.targetToken
            );
        }

        RevertOptions memory revertOptionsUniversal = RevertOptions(
            address(this),
            true,
            callParams.revertOptions.abortAddress,
            abi.encode(
                callParams.revertOptions,
                zrc20,
                callParams.revertOptions.onRevertGasLimit,
                callParams.receiver,
                callParams.data,
                gasZRC20
            ),
            callParams.gasLimit
        );

        bytes memory m = abi.encode(
            callParams.data,
            context.sender,
            outputAmount,
            true,
            context.chainID,
            asset
        );

        emit MessageRelayed();
        gateway.withdrawAndCall(
            callParams.receiver,
            gasZRC20 == callParams.targetToken
                ? outputAmount - gasFee
                : outputAmount,
            callParams.targetToken,
            m,
            CallOptions(callParams.gasLimit, false),
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
        (address gasZRC20, uint256 gasFee) = IZRC20(destination)
            .withdrawGasFeeWithGasLimit(onRevertGasLimit);

        uint256 amountToSwap = context.amount;

        if (gasZRC20 != destination) {
            uint256 inputForGas = SwapHelperLib.swapTokensForExactTokens(
                uniswapRouter,
                context.asset,
                gasFee,
                gasZRC20,
                context.amount
            );
            amountToSwap = context.amount - inputForGas;
            if (!IZRC20(gasZRC20).approve(address(gateway), gasFee)) {
                revert ApproveFailed();
            }
        }

        uint256 out = SwapHelperLib.swapExactTokensForTokens(
            uniswapRouter,
            context.asset,
            amountToSwap,
            destination,
            0
        );
        if (!IZRC20(destination).approve(address(gateway), out)) {
            revert ApproveFailed();
        }

        if (out < gasFee) revert InsufficientOutAmount(out, gasFee);
        gateway.withdrawAndCall(
            abi.encodePacked(revertOptions.revertAddress),
            gasZRC20 == destination ? out - gasFee : out,
            destination,
            abi.encode(data, receiver, out - gasFee, false, targetChainZRC20),
            CallOptions(onRevertGasLimit, false),
            RevertOptions(address(0), false, address(0), "", 0)
        );
    }

    function _decode(
        bytes calldata payload
    ) internal pure returns (CallParams memory p) {
        (p.receiver, p.targetToken, p.data, p.gasLimit, p.revertOptions) = abi
            .decode(payload, (bytes, address, bytes, uint256, RevertOptions));
    }
}
