// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {ERC20BurnableUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20BurnableUpgradeable.sol";
import {ERC20PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PausableUpgradeable.sol";
import {ERC20PermitUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {ERC20BurnableUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20BurnableUpgradeable.sol";

import "../shared/UniversalTokenEvents.sol";

contract UniversalToken is
    Initializable,
    ERC20Upgradeable,
    ERC20BurnableUpgradeable,
    ERC20PausableUpgradeable,
    OwnableUpgradeable,
    ERC20PermitUpgradeable,
    UUPSUpgradeable,
    UniversalContract,
    UniversalTokenEvents
{
    bool public constant isUniversal = true;

    GatewayZEVM public gateway;
    address public uniswapRouter;
    uint256 private _nextTokenId;
    uint256 public gasLimitAmount;

    error TransferFailed();
    error Unauthorized();
    error InvalidAddress();
    error InvalidGasLimit();
    error ApproveFailed();
    error ZeroMsgValue();

    mapping(address => address) public connected;

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address initialOwner,
        string memory name,
        string memory symbol,
        address payable gatewayAddress,
        uint256 gas,
        address uniswapRouterAddress
    ) public initializer {
        __ERC20_init(name, symbol);
        __ERC20Burnable_init();
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        if (gatewayAddress == address(0) || uniswapRouterAddress == address(0))
            revert InvalidAddress();
        if (gas == 0) revert InvalidGasLimit();
        gateway = GatewayZEVM(gatewayAddress);
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
        address destination,
        address receiver,
        uint256 amount
    ) public payable whenNotPaused {
        if (msg.value == 0) revert ZeroMsgValue();
        if (receiver == address(0)) revert InvalidAddress();
        _burn(msg.sender, amount);

        (address gasZRC20, uint256 gasFee) = IZRC20(destination)
            .withdrawGasFeeWithGasLimit(gasLimitAmount);
        if (destination != gasZRC20) revert InvalidAddress();

        address WZETA = gateway.zetaToken();

        IWETH9(WZETA).deposit{value: msg.value}();
        IWETH9(WZETA).approve(uniswapRouter, msg.value);

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

        bytes memory message = abi.encode(receiver, amount, 0, msg.sender);

        CallOptions memory callOptions = CallOptions(gasLimitAmount, false);

        RevertOptions memory revertOptions = RevertOptions(
            address(this),
            true,
            address(0),
            abi.encode(amount, msg.sender),
            gasLimitAmount
        );

        IZRC20(gasZRC20).approve(address(gateway), gasFee);
        gateway.call(
            abi.encodePacked(connected[destination]),
            destination,
            message,
            callOptions,
            revertOptions
        );
        emit TokenTransfer(destination, receiver, amount);
    }

    function mint(address to, uint256 amount) public onlyOwner whenNotPaused {
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

    function pause() public onlyOwner {
        _pause();
    }

    function unpause() public onlyOwner {
        _unpause();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    receive() external payable {}

    // The following functions are overrides required by Solidity.

    function _update(
        address from,
        address to,
        uint256 value
    ) internal override(ERC20Upgradeable, ERC20PausableUpgradeable) {
        super._update(from, to, value);
    }
}
