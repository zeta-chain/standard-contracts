// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "../shared/UniversalNFTEvents.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {ERC721URIStorageUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IWZETA.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";

abstract contract UniversalNFTCore is
    UniversalContract,
    ERC721Upgradeable,
    ERC721URIStorageUpgradeable,
    OwnableUpgradeable,
    UniversalNFTEvents
{
    GatewayZEVM public gateway;
    address public uniswapRouter;
    bool public constant isUniversal = true;
    uint256 public gasLimitAmount;

    mapping(address => address) public connected;

    error TransferFailed();
    error Unauthorized();
    error InvalidAddress();
    error InvalidGasLimit();
    error ApproveFailed();
    error ZeroMsgValue();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    function __UniversalNFTCore_init(
        address gatewayAddress,
        uint256 gas,
        address uniswapRouterAddress
    ) internal {
        if (gatewayAddress == address(0) || uniswapRouterAddress == address(0))
            revert InvalidAddress();
        if (gas == 0) revert InvalidGasLimit();
        gateway = GatewayZEVM(payable(gatewayAddress));
        uniswapRouter = uniswapRouterAddress;
        gasLimitAmount = gas;
    }

    function setGasLimit(uint256 gas) external onlyOwner {
        if (gas == 0) revert InvalidGasLimit();
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
        uint256 tokenId,
        address receiver,
        address destination
    ) public payable {
        if (msg.value == 0) revert ZeroMsgValue();
        if (receiver == address(0)) revert InvalidAddress();

        string memory uri = tokenURI(tokenId);
        _burn(tokenId);

        emit TokenTransfer(receiver, destination, tokenId, uri);

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

        bytes memory message = abi.encode(
            receiver,
            tokenId,
            uri,
            0,
            msg.sender
        );
        CallOptions memory callOptions = CallOptions(gasLimitAmount, false);

        RevertOptions memory revertOptions = RevertOptions(
            address(this),
            true,
            address(0),
            abi.encode(tokenId, uri, msg.sender),
            gasLimitAmount
        );

        if (!IZRC20(gasZRC20).approve(address(gateway), gasFee)) {
            revert ApproveFailed();
        }

        gateway.call(
            abi.encodePacked(connected[destination]),
            destination,
            message,
            callOptions,
            revertOptions
        );
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
            uint256 tokenId,
            string memory uri,
            address sender
        ) = abi.decode(message, (address, address, uint256, string, address));

        if (destination == address(0)) {
            _safeMint(receiver, tokenId);
            _setTokenURI(tokenId, uri);
            emit TokenTransferReceived(receiver, tokenId, uri);
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
                abi.encode(receiver, tokenId, uri, out - gasFee, sender),
                CallOptions(gasLimitAmount, false),
                RevertOptions(
                    address(this),
                    true,
                    address(0),
                    abi.encode(tokenId, uri, sender),
                    0
                )
            );
        }
        emit TokenTransferToDestination(receiver, destination, tokenId, uri);
    }

    function onRevert(RevertContext calldata context) external onlyGateway {
        (uint256 tokenId, string memory uri, address sender) = abi.decode(
            context.revertMessage,
            (uint256, string, address)
        );

        _safeMint(sender, tokenId);
        _setTokenURI(tokenId, uri);
        emit TokenTransferReverted(sender, tokenId, uri);
    }

    function tokenURI(
        uint256 tokenId
    )
        public
        view
        virtual
        override(ERC721Upgradeable, ERC721URIStorageUpgradeable)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(
        bytes4 interfaceId
    )
        public
        view
        virtual
        override(ERC721Upgradeable, ERC721URIStorageUpgradeable)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
