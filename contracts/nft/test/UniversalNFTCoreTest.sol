// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/ZRC20.sol";
import "@zetachain/protocol-contracts/contracts/zevm/SystemContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import {Upgrades} from "openzeppelin-foundry-upgrades/Upgrades.sol";
import "./utils/WZETA.sol";
import "../contracts/zetachain/UniversalNFT.sol";
import "../contracts/shared/UniversalNFTEvents.sol";

import "@uniswap/v2-core/contracts/interfaces/IUniswapV2Factory.sol";
import "@uniswap/v2-periphery/contracts/interfaces/IUniswapV2Router02.sol";

contract UniversalNFTIntegrationTest is
    Test,
    IGatewayZEVMEvents,
    UniversalNFTEvents
{
    address payable proxy;
    GatewayZEVM gateway;
    ZRC20 zrc20;
    WETH9 zetaToken;
    SystemContract systemContract;
    address owner;
    address addr1;
    address protocolAddress;
    RevertOptions revertOptions;
    CallOptions callOptions;

    function setUp() public {
        IUniswapV2Factory factory;
        IUniswapV2Router02 router;
        address zrc20AndZetaPair;

        // factory = new UniswapV2Factory(owner);
        factory = IUniswapV2Factory(
            deployCode(
                "dependencies/@uniswap-v2-core-1.0.1/contracts/UniswapV2Factory.sol"
            )
        );

        // router = new UniswapV2Router02(
        //     address(factory),
        //     address(zetaToken) // WETH9-compatible token
        // );

        owner = address(this);
        addr1 = address(0x1234);

        zetaToken = new WETH9();

        proxy = payable(
            Upgrades.deployUUPSProxy(
                "GatewayZEVM.sol",
                abi.encodeCall(
                    GatewayZEVM.initialize,
                    (address(zetaToken), owner)
                )
            )
        );
        gateway = GatewayZEVM(proxy);

        protocolAddress = gateway.PROTOCOL_ADDRESS();

        vm.startPrank(protocolAddress);
        systemContract = new SystemContract(address(0), address(0), address(0));
        zrc20 = new ZRC20(
            "TOKEN",
            "TKN",
            18,
            1,
            CoinType.Gas,
            0,
            address(systemContract),
            address(gateway)
        );
        systemContract.setGasCoinZRC20(1, address(zrc20));
        systemContract.setGasPrice(1, 1);
        vm.deal(protocolAddress, 1_000_000_000);
        zetaToken.deposit{value: 10}();
        zetaToken.approve(address(gateway), 10);
        zrc20.deposit(owner, 100_000);
        vm.stopPrank();

        vm.startPrank(owner);
        zrc20.approve(address(gateway), 100_000);
        zetaToken.deposit{value: 10}();
        zetaToken.approve(address(gateway), 10);
        vm.stopPrank();

        revertOptions = RevertOptions({
            revertAddress: address(0x321),
            callOnRevert: true,
            abortAddress: address(0x321),
            revertMessage: "",
            onRevertGasLimit: 0
        });

        callOptions = CallOptions({gasLimit: 1, isArbitraryCall: true});
    }

    function testTransferCrossChain() public {
        // Arrange
        uint256 tokenId = 1;
        address receiver = addr1;
        address destination = address(zrc20);

        UniversalNFT nftContract = new UniversalNFT();
        nftContract.initialize(
            address(this),
            "TestNFT",
            "TNFT",
            payable(address(gateway)),
            100000,
            0xF62849F9A0B5Bf2913b396098F7c7019b51A820a
        );

        nftContract.safeMint(owner, "ipfs://test-uri");

        // Act
        vm.startPrank(owner);
        nftContract.transferCrossChain{value: 1 ether}(
            tokenId,
            receiver,
            destination
        );
        vm.stopPrank();

        // Assert
        vm.expectEmit(true, true, true, true);
        emit TokenTransfer(receiver, destination, tokenId, "ipfs://test-uri");

        assertEq(nftContract.ownerOf(tokenId), address(0));
    }
}