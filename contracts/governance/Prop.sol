// SPDX-License-Identifier: MIT

import "../Idea.sol";
import "./Funding.sol";

pragma solidity ^0.8.11;

/**
 * Represents an ongoing vote to implement a new funding rate for an idea.
 * Votes are weighted based on balances held.
 */
contract Prop {
	/* The idea constituting the voting body */
	Idea public governed;

	/* The idea being funded by the prop */
	Idea public toFund;

	/* The weighted average rate so far */
	FundingRate public rate;

	/* The number of days that the vote lasts */
	uint256 public expiresAt;

	/* A new proposal was created, the details of which are on IPFS */
	event NewProposal(Idea governed, Idea toFund, string propIpfsHash, uint256 expiresAt);

	/**
	 * Creates a new proposal, whose details should be on IPFS already, and that
	 * expires at the indicated time.
	 *
	 * @param _jurisdiction - The token measuring votes
	 * @param _toFund - The idea whose funding is being voted on
	 * @param _token - The token being used to fund the idea
	 * @param _fundingType - How the reward should be fundraised (i.e., minting or from the treasury)
	 * @param _proposalIpfsHash - The details of the proposal, in any form, available
	 * on IPFS
	 * @param _expiry - The number of days that the vote can last for
	 */
	constructor(Idea _jurisdiction, Idea _toFund, address _token, FundingType _fundingType, string memory _proposalIpfsHash, uint256 _expiry) {
		governed = _jurisdiction;
		toFund = _toFund;
		rate = FundingRate(_token, 0, 0, 0, 0, _fundingType);
		expiresAt = block.timestamp + _expiry * 1 days;

		emit NewProposal(_jurisdiction, _toFund, _proposalIpfsHash, expiresAt);
	}

	/**
	 * Delegates the specified number of votes (tokens) to this proposal with
	 * the given vote details.
	 */
	function vote(uint256 _votes, FundingRate calldata _rate) external {
		require(governed.transferFrom(msg.sender, address(this), _votes), "Failed to delegate votes");

		// Votes have to be weighted by their balance of the governing token
		uint256 weight = _votes;

		rate.value += weight * _rate.value;
		rate.intervalLength += weight * _rate.intervalLength;
		rate.expiry += weight * _rate.expiry;
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
