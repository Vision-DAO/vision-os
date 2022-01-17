import "./governance/Funding.sol";
import "./governance/Prop.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

pragma solidity ^0.8.11;

/* Represents the governance contract / semi-fungible NFT of an idea in the
 * value tree. */
contract Idea is ERC20 {
	/* Funding rates for derivative ideas */
	public mapping (address => FundingRate) fundedIdeas;

	/* The idea, and its datum have been committed to the blockchain. */
	event IdeaRecorded(string ipfsAddr);

	/* A child idea has had a new funds rate finalized. */
	event IdeaFunded(address idea, FundsRate rate);

	/* An instance of a child's funding has been released. */
	event FundingDispersed(address idea, FundsRate rate);

	modifier isChild {
		FundingRate rate = this.fundedIdeas[msg.sender];

		// The governing contract has to have funds left in the designated token to transfer to the child
		require(rate.value > 0 && ((rate.token == 0x0 && this.balance >= rate.value) || ((rate.token.kind == FundingType.MINT && rate.token == this) || rate.token.balanceOf(this) > rate.value)), "No funds to allocate");
		;
	}

	/**
	 * Creates a new idea from the given datum stored on IPFS, and idea token attributes.
	 */
	constructor(string ideaName, string ideaTicker, uint256 ideaShares, string datumIpfsHash) ERC20(ideaName, ideaTicker) {
		_mint(ideaShares, msg.sender);

		emit IdeaRecorded(datumIpfsHash);
	}

	/**
	 * Finalizes the given proposition if it has past its expiry date.
	 */
	function finalizeProp(Prop proposal) external {
		require(block.timestamp >= proposal.expiresAt, "Vote has not yet terminated.");

		// Votes for the funds rate are weighted based on balances of this governing
		// token
		FundingRate finalRate = proposal.finalFundsRate();
		finalRate.value = (finalRate.expiry - block.timestamp) / finalRate.intervalLength;

		// Record the new funds rate
		this.fundedIdeas[proposal.toFund] = finalRate;
		emit IdeaFunded(proposal.toFund, finalRate);
	}

	/**
	 * Disperses funding to the calling Idea, if it is a child in the
	 * jurisdiction of the current token, and has funds to be allocated.
	 */
	function disperseFunding() external isChild {
		FundingRate rate = this.fundedIdeas[msg.sender];

		require(rate.expiry < block.timestamp, "Funding has expired");
		require(block.timestamp - rate.lastClaimed >= rate.intervalLength, "Claimed too recently");

		this.fundedIdeas[msg.sender].lastClaimed = block.timestamp;

		// The number of tokens to disperse
		uint256 tokens = rate.value;

		// The idea can be rewarded in ETH or an ERC-20
		if (this.fundedIdeas[msg.sender].token == 0x0) {
			require(msg.sender.call{value: tokens}(""), "Failed to disperse ETH rewards");
		} else {
			// If the reward is in our own token, mint it
			if (rate.token == this && rate.kind == FundingType.MINT) {
				_mint(tokens, msg.sender);
			} else {
				require(rate.token.transfer(msg.sender, tokens), "Failed to disperse ERC rewards");
			}
		}

		emit FundingDispersed(msg.sender, rate);
	}
}
