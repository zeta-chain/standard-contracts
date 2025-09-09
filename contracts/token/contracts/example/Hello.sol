// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import {ERC721Holder} from "@openzeppelin/contracts/token/ERC721/utils/ERC721Holder.sol";

contract Hello is ERC721Holder {
    event MessageReceived(bytes);

    function hello(bytes memory message) external {
        emit MessageReceived(message);
    }
}
