// SPDX-License-Identifier: MIT

import "../Idea.sol";
import "./Funding.sol";

pragma solidity ^0.8.11;

/**
 * Details of individual votes must be stored to allow "undo" functionality.
 */
struct Vote {
	FundingRate rate;
	uint256 votes;
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

	/* The weighted average rate so far */
	FundingRate public rate;

	/* Users that voted on the proposal - should receive a refund after */
	mapping (address => Vote) public refunds;

	/* Where metadata about the proposal is stored */
	string public ipfsAddr;

	uint256 public nVoters;

	/* The title of the proposal */
	string public title;

	address[] public voters;

	/* The number of seconds that the vote lasts */
	uint256 public expiresAt;

	/* A new proposal was created, the details of which are on IPFS */
	event NewProposal(Idea governed, address toFund, string propIpfsHash, uint256 expiresAt);

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
	 * @param _expiry - The number of seconds that the vote can last for
	 */
	constructor(string memory _propName, Idea _jurisdiction, address _toFund, address _token, FundingType _fundingType, string memory _proposalIpfsHash, uint256 _expiry) {
		title = _propName;
		governed = _jurisdiction;
		toFund = _toFund;
		rate = FundingRate(_token, 0, 0, 0, 0, _fundingType);
		expiresAt = block.timestamp + _expiry * 1 seconds;
		ipfsAddr = _proposalIpfsHash;

		emit NewProposal(_jurisdiction, _toFund, _proposalIpfsHash, expiresAt);
	}

	/**
	 * Delegates the specified number of votes (tokens) to this proposal with
	 * the given vote details.
	 */
	function vote(uint256 _votes, FundingRate calldata _rate) external {
		require(_votes > 0, "No votes to delegate");
		require(governed.transferFrom(msg.sender, address(this), _votes), "Failed to delegate votes");

		// Votes have to be weighted by their balance of the governing token
		uint256 weight = _votes;

		rate.value += weight * _rate.value;
		rate.intervalLength += weight * _rate.intervalLength;
		rate.expiry += weight * _rate.expiry;

		// Voters should be able to get their tokens back after the vote is over
		// Register the voter for a refund when the proposal expires
		if (refunds[msg.sender].votes == 0) {
			voters.push(msg.sender);
			nVoters++;
		}

		refunds[msg.sender] = Vote(_rate, _votes);
	}

	/**
	 * Deallocates all votes from the user.
	 */
	function refundVotes() external {
		require(refunds[msg.sender].votes > 0, "No votes left to refund.");

		uint256 w = refunds[msg.sender].votes;
		FundingRate memory r = refunds[msg.sender].rate;

		// Refund the user
		require(governed.transfer(msg.sender, w), "Failed to refund votes");

		// Subtract their weighted votes from the total
		rate.value -= w * r.value;
		rate.intervalLength -= w * r.intervalLength;
		rate.expiry -= w * r.expiry;

		// Remove the user's refund entry
		delete refunds[msg.sender];

		for (uint256 i = 0; i < nVoters; i++) {
			if (voters[i] == msg.sender) {
				voters[i] = voters[nVoters - 1];
				voters.pop();

				break;
			}
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

	/**
	 * Calculates the weighted average of the funds rate varaibles, returning
	 * the resultant funds rate, without an updated finalization date.
	 */
	function finalFundsRate() view external returns (FundingRate memory) {
		// The total number of votes is the total number of tokens delegated to this account
		uint256 totalVotes = governed.balanceOf(address(this));

		FundingRate memory finalRate = rate;

		finalRate.value /= totalVotes;
		finalRate.intervalLength /= totalVotes;
		finalRate.expiry /= totalVotes;

		return finalRate;
	}
}
