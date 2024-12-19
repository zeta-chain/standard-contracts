// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IWZETA.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import {SwapHelperLib} from "@zetachain/toolkit/contracts/SwapHelperLib.sol";
import {SystemContract} from "@zetachain/toolkit/contracts/SystemContract.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {ERC721Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol";
import {ERC721EnumerableUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721EnumerableUpgradeable.sol";
import {ERC721URIStorageUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721URIStorageUpgradeable.sol";
import {ERC721BurnableUpgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/ERC721BurnableUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import "../shared/Events.sol";

contract UniversalNFT is
    Initializable,
    ERC721Upgradeable,
    ERC721EnumerableUpgradeable,
    ERC721URIStorageUpgradeable,
    ERC721BurnableUpgradeable,
    OwnableUpgradeable,
    UniversalContract,
    UUPSUpgradeable,
    Events
{
    GatewayZEVM public gateway;
    address public uniswapRouter;
    uint256 private _nextTokenId;
    bool public constant isUniversal = true;
    uint256 public gasLimitAmount;

    bool public allowOutgoing;
    bool public allowIncoming;

    error TransferFailed();
    error Unauthorized();
    error InvalidAddress();
    error InvalidGasLimit();
    error ApproveFailed();
    error OutgoingTransfersNotAllowed();
    error IncomingTransfersNotAllowed();

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
        __ERC721_init(name, symbol);
        __ERC721Enumerable_init();
        __ERC721URIStorage_init();
        __ERC721Burnable_init();
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        if (gatewayAddress == address(0) || uniswapRouterAddress == address(0))
            revert InvalidAddress();
        if (gas == 0) revert InvalidGasLimit();
        gateway = GatewayZEVM(gatewayAddress);
        uniswapRouter = uniswapRouterAddress;
        gasLimitAmount = gas;
        allowOutgoing = true;
        allowIncoming = true;
    }

    function setAllowOutgoing(bool _allowOutgoing) external onlyOwner {
        allowOutgoing = _allowOutgoing;
    }

    function setAllowIncoming(bool _allowIncoming) external onlyOwner {
        allowIncoming = _allowIncoming;
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
        if (!allowOutgoing) revert OutgoingTransfersNotAllowed();
        if (receiver == address(0)) revert InvalidAddress();

        string memory uri = tokenURI(tokenId);
        _burn(tokenId);

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

        IZRC20(gasZRC20).approve(address(gateway), gasFee);
        gateway.call(
            abi.encodePacked(connected[destination]),
            destination,
            message,
            callOptions,
            revertOptions
        );
    }

    function safeMint(address to, string memory uri) public onlyOwner {
        uint256 hash = uint256(
            keccak256(
                abi.encodePacked(address(this), block.number, _nextTokenId++)
            )
        );

        uint256 tokenId = hash & 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

        _safeMint(to, tokenId);
        _setTokenURI(tokenId, uri);
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
            if (!allowIncoming) revert IncomingTransfersNotAllowed();
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

    // The following functions are overrides required by Solidity.

    function _update(
        address to,
        uint256 tokenId,
        address auth
    )
        internal
        override(ERC721Upgradeable, ERC721EnumerableUpgradeable)
        returns (address)
    {
        return super._update(to, tokenId, auth);
    }

    function _increaseBalance(
        address account,
        uint128 value
    ) internal override(ERC721Upgradeable, ERC721EnumerableUpgradeable) {
        super._increaseBalance(account, value);
    }

    function tokenURI(
        uint256 tokenId
    )
        public
        view
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
        override(
            ERC721Upgradeable,
            ERC721EnumerableUpgradeable,
            ERC721URIStorageUpgradeable
        )
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    receive() external payable {}
}
