// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "./Idea.sol";
import "./Proposal.sol";

contract RecoveryGroup is Idea {
    /* The parent of the user. */
    address internal parentUser;

    /* The current proposal, if any. */
    RecoveryProp public currProp;

    constructor() Idea("", "IJK", 0, "") {
        parentUser = msg.sender;
    }

    function addTrustedContact(address memory contact) external {
        require(msg.sender == parentUser, "Trusted contacts can only be added by owner");

        mint(contact, 1 * 10**19);
    }

    function removeTrustedContact(address memory contact) external {
        require(msg.sender == parentUser, "Trusted contacts can only be removed by owner");

        burnFrom(contact, 1 * 10**19);
    }

    function requestRecovery(address newIdent) external {
        require(currProp == address(0), "A current recovery session may not be overwritten");

        // 24 hour duration
        currProp = RecoveryProp(newIdent, 24 * 60 * 60, msg.sender);
    }

    function recover() external returns (address) {
        require(currProp == address(0), "No recovery proposal is active");
        require(!currProp.isActive(), "The recovery proposal is still pending");

        // Require a >50% majority to pass any proposal
        if (currProp.nAffirmative() * 100 / totalSupply() <= 50) {
            emit ProposalRejected(proposal);

            return address(0);
        }

        address newIdent = currProp.newIdent;
        currProp = address(0);
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
