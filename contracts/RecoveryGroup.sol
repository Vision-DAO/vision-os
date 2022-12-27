// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./Idea.sol";
import "./Proposal.sol";
import "./User.sol";

contract RecoveryGroup is Idea {
    /* The parent of the user. */
    User internal parentUser;

    /* The current proposal, if any. */
    RecoveryProp public currProp;

    constructor() Idea("", "IJK", 0, "") {
        parentUser = User(msg.sender);
    }

    function addTrustedContact(address contact) external {
        require(msg.sender == parentUser.ident(), "Trusted contacts can only be added by owner");

        _mint(contact, 1 * 10**19);
    }

    function removeTrustedContact(address contact) external {
        require(msg.sender == parentUser.ident(), "Trusted contacts can only be removed by owner");

        _burn(contact, 1 * 10**19);
    }

    function requestRecovery(address newIdent) external {
        require(address(currProp) == address(0), "A current recovery session may not be overwritten");

        // 24 hour duration
        currProp = new RecoveryProp(newIdent, 24 * 60 * 60, this);
    }

    function recover() external returns (address) {
        require(address(currProp) == address(0), "No recovery proposal is active");
        require(!currProp.active(), "The recovery proposal is still pending");

        // Require a >50% majority to pass any proposal
        if (currProp.nAffirmative() * 100 / totalSupply() <= 50) {
            emit ProposalRejected(currProp);

            return address(0);
        }

        address newIdent = currProp.newIdent();
        currProp = RecoveryProp(address(0));
        return newIdent;
    }
}

contract RecoveryProp is Proposal {

    /* The new identity to be voted on. */
    address public newIdent;
    /**
     * Creates a new proposal with the given metadata, voting period length,
     * and parent contract. The parent contract must be an instance of the Idea
     * contract.
     * @param _newIdent - The new identity to be voted on.
     * @param _duration - The number of seconds that the voting period will
     *  last, after it has begun
     * @param _governor - The address of the contract whose tokens represent
     *  votes in either direction for the proposal.
     */
    constructor(address _newIdent, uint256 _duration, Idea _governor) Proposal(_duration, _governor) {
        newIdent = _newIdent;
    }
}
