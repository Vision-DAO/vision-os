// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

interface IHasMetadata {
    function ipfsAddr() external view returns (string memory);
}
