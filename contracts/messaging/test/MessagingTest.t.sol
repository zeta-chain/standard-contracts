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
            address(zetaSetup.uniswapV2Router())
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
        ethMessaging.setConnected(
            bnb_bnb.zrc20,
            abi.encodePacked(address(bnbMessaging))
        );
        bnbMessaging.setConnected(
            eth_eth.zrc20,
            abi.encodePacked(address(ethMessaging))
        );
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
            bnb_bnb.zrc20,
            testData,
            gasLimit,
            revertOptions
        );
        vm.stopPrank();
    }

    // function test_sendMessage_revert_uint256() public {
    //     // Send a uint256 as testData
    //     uint256 value = 12345;
    //     bytes memory testData = abi.encode(value);
    //     uint256 gasLimit = 100000;
    //     // Set callOnRevert to true and revertAddress to ethMessaging
    //     RevertOptions memory revertOptions = RevertOptions({
    //         revertAddress: address(ethMessaging),
    //         callOnRevert: true,
    //         abortAddress: address(0),
    //         revertMessage: "",
    //         onRevertGasLimit: 100000
    //     });

    //     // Expect the MessageRelayed event from the router
    //     vm.expectEmit(false, false, false, true, address(router));
    //     emit UniversalRouter.MessageRelayed();

    //     // Expect the OnRevertEvent from ethMessaging
    //     vm.expectEmit(false, false, false, false, address(ethMessaging));
    //     emit Example.OnMessageRevertEvent();

    //     vm.prank(alice);
    //     ethMessaging.sendMessage{value: 1 ether}(
    //         bnb_bnb.zrc20,
    //         testData,
    //         gasLimit,
    //         revertOptions
    //     );
    //     vm.stopPrank();
    // }
}
