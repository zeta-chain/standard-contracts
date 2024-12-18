// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

import "../shared/Events.sol";

contract UniversalToken is
    Initializable,
    ERC20Upgradeable,
    OwnableUpgradeable,
    UUPSUpgradeable,
    Events
{
    GatewayEVM public gateway;
    address public universal;
    uint256 public gasLimitAmount;

    error InvalidAddress();
    error Unauthorized();
    error InvalidGasLimit();

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
        uint256 gas
    ) public initializer {
        __ERC20_init(name, symbol);
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        if (gatewayAddress == address(0)) revert InvalidAddress();
        gasLimitAmount = gas;
        gateway = GatewayEVM(gatewayAddress);
    }

    function setUniversal(address contractAddress) external onlyOwner {
        if (contractAddress == address(0)) revert InvalidAddress();
        universal = contractAddress;
        emit SetUniversal(contractAddress);
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
            if (!success) emit RefundFailed(sender, gasAmount);
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

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}
