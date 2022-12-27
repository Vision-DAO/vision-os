// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./Idea.sol";
import "./RecoveryGroup.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

contract User is Idea {
    /* Token whose owners are responsible for allocation of governance token. */
    RecoveryGroup public recoveryGroup;
    /* Encrypted public-private key pair that can authenticate the user. */
    string public keyPair;
    /* Address of the user */
    address public ident;

    modifier isOwner {
        require(msg.sender == ident, "Must be owner to perform this action");
        _;
    }

    constructor(string memory _keyPair, string memory _ipfsAddr) Idea(Strings.toHexString(uint160(msg.sender), 20), "XYZ", 1 * 10**19, _ipfsAddr) {
        recoveryGroup = new RecoveryGroup();
        keyPair = _keyPair;
        ident = msg.sender;
    }

    function addTrustedContact(address contact) external isOwner {
        recoveryGroup.addTrustedContact(contact);
    }

    function removeTrustedContact(address contact) external isOwner {
        recoveryGroup.removeTrustedContact(contact);
    }

    function recover() external {
        require(address(recoveryGroup.currProp()) != address(0), "There is no active recovery proposal");
        address newIdent = recoveryGroup.recover();
        require(newIdent != address(0), "Recovery proposal did not pass");

        _burn(ident, 1 * 10**19);
        ident = newIdent;
        _mint(ident, 1 * 10**19);
    }

    function setKeyPair(string calldata newKeyPair) external isOwner {
        keyPair = newKeyPair;
    }

    function transfer(address token, address recipient, uint256 amount) external isOwner {
        require(ERC20(token).transfer(recipient, amount), "Transfer failed");
    }

    function setMetadata(string memory _ipfsAddr) external isOwner {
        ipfsAddr = _ipfsAddr;
    }
}
