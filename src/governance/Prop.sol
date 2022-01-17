import "../Idea.sol";

pragma solidity ^0.8.11;

/**
 * Represents an ongoing vote to implement a new funding rate for an idea.
 * Votes are weighted based on balances held.
 */
contract Prop {
	/* The idea constituting the voting body */
	Idea governed;

	/* The idea being funded by the prop */
	Idea toFund;

	/* The weighted average rate so far */
	FundingRate rate;

	/* The number of days that the vote lasts */
	uint256 expiresAt;

	/* A new proposal was created, the details of which are on IPFS */
	event NewProposal(Idea governed, Idea toFund, string propIpfsHash, uint256 expiresAt);

	/**
	 * Creates a new proposal, whose details should be on IPFS already, and that
	 * expires at the indicated time.
	 *
	 * @param jurisdiction - The token measuring votes
	 * @param toFund - The idea whose funding is being voted on
	 * @param token - The token being used to fund the idea
	 * @param fundingKind - How the reward should be fundraised (i.e., minting or from the treasury)
	 * @param proposalIpfsHash - The details of the proposal, in any form, available
	 * on IPFS
	 * @param expiry - The number of days that the vote can last for
	 */
	constructor(Idea jurisdiction, Idea toFund, address token, FundingKind fundingType, string proposalIpfsHash, uint256 expiry) {
		this.governed = jurisdiction;
		this.toFund = toFund;
		this.rate = FundingRate(token, 0, 0, 0, 0, fundingType);
		this.expiresAt = block.timestamp + expiry days;

		emit NewProposal(jurisdiction, toFund, proposalIpfsHash, this.expiresAt);
	}

	/**
	 * Delegates the specified number of votes (tokens) to this proposal with
	 * the given vote details.
	 */
	function vote(uint256 votes, FundingRate rate) external {
		require(governed.transferFrom(msg.sender, this, votes), "Failed to delegate votes");

		// Votes have to be weighted by their balance of the governing token
		uint256 weight = governed.balanceOf(msg.sender);

		this.rate.value += weight * rate.value;
		this.rate.intervalLength += weight * rate.intervalLength;
		this.rate.expiry += weight * rate.expiry;
	}

	/**
	 * Calculates the weighted average of the funds rate varaibles, returning
	 * the resultant funds rate, without an updated finalization date.
	 */
	function finalFundsRate() pure external returns (FundingRate) {
		// The total number of votes is the total number of tokens delegated to this account
		uint256 totalVotes = governed.balanceOf(this);

		this.rate.value /= totalVotes;
		this.rate.intevalLength /= totalVotes;
		this.rate.expiry /= totalVotes;

		return this.rate;
	}
}
