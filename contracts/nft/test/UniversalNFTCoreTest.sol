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
    address receiver;
    address protocolAddress;
    RevertOptions revertOptions;
    CallOptions callOptions;
    IUniswapV2Factory factory;
    IUniswapV2Router02 router;

    function setUp() public {
        owner = address(this);
        receiver = address(0x1234);

        vm.deal(owner, 100 ether);

        zetaToken = new WETH9();

        factory = IUniswapV2Factory(
            deployCode(
                "out/UniswapV2Factory.sol/UniswapV2Factory.json",
                abi.encode(address(0x1))
            )
        );

        router = IUniswapV2Router02(
            deployCode(
                "out/UniswapV2Router02.sol/UniswapV2Router02.json",
                abi.encode(address(factory), address(zetaToken))
            )
        );

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

        vm.deal(protocolAddress, 100 ether);
        zetaToken.deposit{value: 10 ether}();
        zetaToken.approve(address(gateway), 10 ether);

        zrc20.deposit(owner, 100_000 * 1e18);

        vm.stopPrank();

        vm.startPrank(owner);

        zrc20.approve(address(gateway), 100_000 * 1e18);

        zetaToken.deposit{value: 10 ether}();
        zetaToken.approve(address(gateway), 10 ether);

        zetaToken.approve(address(router), 10 ether);
        zrc20.approve(address(router), 10 ether);

        factory.createPair(address(zetaToken), address(zrc20));

        router.addLiquidity(
            address(zetaToken),
            address(zrc20),
            10 ether,
            10 ether,
            1 ether,
            1 ether,
            owner,
            block.timestamp + 300
        );

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
        address destination = address(zrc20);
        string memory uri = "ipfs://test-uri";

        address proxyAddr = Upgrades.deployUUPSProxy(
            "ZetaChainUniversalNFT.sol",
            abi.encodeCall(
                UniversalNFT.initialize,
                (
                    owner,
                    "TestNFT",
                    "TNFT",
                    payable(address(gateway)),
                    100_000,
                    address(router)
                )
            )
        );

        UniversalNFT nftContract = UniversalNFT(payable(proxyAddr));

        uint256 tokenId = nftContract.safeMint(receiver, uri);

        assertEq(nftContract.ownerOf(tokenId), receiver);
        assertEq(nftContract.tokenURI(tokenId), uri);

        // vm.startPrank(owner);
        // nftContract.transferCrossChain{value: 1 ether}(
        //     tokenId,
        //     receiver,
        //     destination
        // );
        // vm.stopPrank();

        // vm.expectEmit(true, true, true, true);
        // emit TokenTransfer(receiver, destination, tokenId, "ipfs://test-uri");

        // assertEq(nftContract.ownerOf(tokenId), address(0));
    }
}
