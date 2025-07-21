// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.26;

import "@zetachain/toolkit/contracts/testing/FoundrySetup.t.sol";
import {Example} from "../contracts/Example.sol";
import {UniversalRouter} from "../contracts/UniversalRouter.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import {WrapGatewayEVM} from "@zetachain/toolkit/contracts/testing/mockGateway/WrapGatewayEVM.sol";
import {WrapGatewayZEVM} from "@zetachain/toolkit/contracts/testing/mockGateway/WrapGatewayZEVM.sol";
import {NodeLogicMock} from "@zetachain/toolkit/contracts/testing/mockGateway/NodeLogicMock.sol";

contract MessagingTest is FoundrySetup {
    Example public zetaMessaging;
    Example public ethMessaging;
    Example public bnbMessaging;
    UniversalRouter public router;

    address owner = makeAddr("Owner");
    address alice = makeAddr("Alice");
    address bob = makeAddr("Bob");
    address contractRegistry = makeAddr("registry");

    function setUp() public override {
        super.setUp();
        deal(owner, 1_000_000 ether);
        deal(alice, 1_000_000 ether);
        deal(bob, 1_000_000 ether);
        vm.startPrank(owner);

        // Set router address for testing
        router = new UniversalRouter(
            payable(address(zetaSetup.wrapGatewayZEVM())),
            owner,
            address(zetaSetup.uniswapV2Router()),
            contractRegistry
        );

        ethMessaging = new Example(
            payable(address(evmSetup.wrapGatewayEVM(chainIdETH))),
            owner,
            address(router)
        );

        bnbMessaging = new Example(
            payable(address(evmSetup.wrapGatewayEVM(chainIdBNB))),
            owner,
            address(router)
        );
        ethMessaging.setConnected(97, abi.encodePacked(address(bnbMessaging)));
        bnbMessaging.setConnected(5, abi.encodePacked(address(ethMessaging)));
        vm.stopPrank();
    }

    function test_sendMessage() public {
        bytes memory testData = abi.encode("Hello Cross-Chain World!");
        uint256 gasLimit = 100000;
        RevertOptions memory revertOptions = RevertOptions({
            revertAddress: alice,
            callOnRevert: false,
            abortAddress: address(0),
            revertMessage: "",
            onRevertGasLimit: 0
        });

        vm.expectEmit(false, false, false, true, address(router));
        emit UniversalRouter.MessageRelayed();

        vm.expectEmit(false, false, false, true, address(bnbMessaging));
        emit Example.OnMessageReceiveEvent(testData);

        vm.prank(alice);
        ethMessaging.sendMessage{value: 1 ether}(
            abi.encodePacked(address(bnbMessaging)),
            address(bnb_bnb.zrc20),
            testData,
            gasLimit,
            revertOptions
        );
        vm.stopPrank();
    }
}
