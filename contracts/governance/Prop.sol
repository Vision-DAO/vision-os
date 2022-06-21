// SPDX-License-Identifier: MIT

import "../Idea.sol";
import "./Funding.sol";

pragma solidity ^0.8.11;

/**
 * The DAO attributes that a proposal is capable of targeting.
 */
enum PropTarget {
	PAYLOAD,
	FUNDING
}

/**
 * Represents an ongoing vote to implement a new funding rate for an idea.
 * Votes are weighted based on balances held.
 */
contract Prop {
	/* The idea constituting the voting body */
	Idea public governed;

	/* The idea being funded by the prop */
	address public toFund;

	/* The proposed funding rate for the specified address */
	FundingRate public rate;

	/* The proposed new metadata address */
	string public payload;

	/* Whether the Idea's funding rate, payload, or both, were changed */
	mapping (PropTarget => bool) public diff;

	/* Users that voted on the proposal - should receive a refund after */
	mapping (address => uint256) public refunds;

	/* Where metadata about the proposal is stored */
	string public ipfsAddr;

	/* The names of all voters eligible for refunds */
	uint256 public nVoters;
	address[] public voters;

	/* The UNIX timestamp that the proposal shall last until */
	uint256 public expiresAt;

	/* A new proposal was created, the details of which are on IPFS */
	event NewProposal(Idea governed, address toFund, string propIpfsHash, uint256 expiresAt);

	modifier isActive {
		require(block.timestamp < expiresAt, "The proposal is no longer active");
		_;
	}

	/**
	 * Creates a new proposal, whose details should be on IPFS already, and that
	 * expires at the indicated time.
	 *
	 * @param _propName - The title of the proposal
	 * @param _jurisdiction - The token measuring votes
	 * @param _toFund - The idea whose funding is being voted on
	 * @param _token - The token being used to fund the idea
	 * @param _fundingType - How the reward should be fundraised (i.e., minting or from the treasury)
	 * @param _proposalIpfsHash - The details of the proposal, in any form, available
	 * on IPFS
	 * @param _metaPayload - The IPFS address of the new metadata to use for
	 * the governing contract
	 * @param _targetAttrs - Whether the metadata, funding rate, or both, were
	 * changed for the governing DAO
	 * @param _expiry - The UNIX timestamp of the expiration date
	 */
	constructor(string memory _proposalIpfsHash, Idea _jurisdiction, address _toFund, FundingRate _fundingRate, string memory _metaPayload, mapping (PropTarget => bool) _targetAttrs, uint256 _expiry) {
		ipfsAddr = _proposalIpfsHash;
		governed = _jurisdiction;
		payload = _metaPayload;
		newIpfsAddr = _toFund;
		rate = _fundingRate;
		expiresAt = _expiry;

		emit NewProposal(_jurisdiction, _toFund, _proposalIpfsHash, expiresAt);
	}

	/**
	 * Delegates the specified number of votes (tokens) to this proposal with an
	 * affirmative vote.
	 *
	 * pre: The voting period has not passed
	 * pre: The user has a balance greater or equal to the number of votes they
	 * want to cast
	 * pre: The user is casting a non-zero number of votes
	 */
	function vote(uint256 _votes) external isActive {
		require(_votes > 0, "No votes to delegate");
		require(governed.transferFrom(msg.sender, address(this), _votes), "Failed to delegate votes");

		// Voters should be able to get their tokens back after the vote is over
		// Register the voter for a refund when the proposal expires
		if (refunds[msg.sender].votes == 0) {
			voters.push(msg.sender);
			nVoters++;
		}

		refunds[msg.sender] += _votes;
	}

	/**
	 * Deallocates all votes from the user.
	 */
	function refundVotes() external {
		require(refunds[msg.sender].votes > 0, "No votes left to refund");

		uint256 n = refunds[msg.sender];

		// Refund the user
		require(governed.transfer(msg.sender, n), "Failed to refund votes");

		// Remove the user's refund entry
		delete refunds[msg.sender];

		if (nVoters > 1) {
			voters[i] = voters[nVoters - 1];
			delete voters[nVoters - 1];
		}

		nVoters--;
	}

	/**
	 * Refunds token votes to all voters, if the msg.sender is the governing
	 * contract.
	 */
	function refundAll() external returns (bool) {
		// Not any user can call refund
		require(msg.sender == address(governed), "Refunder is not the governor");

		// Refund all voters
		for (uint i = 0; i < nVoters; i++) {
			address voter = address(voters[i]);

			require(governed.transfer(voter, refunds[voter].votes), "Failed to refund all voters");
		}

		// All voters were successfully refunded
		return true;
	}
}
