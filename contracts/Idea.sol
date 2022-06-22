// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "../node_modules/@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./Prop.sol";

/**
 * A contract that implements a DAO for establishing consensus about the state
 * of associated metadata, as defined in the Vision V2 spec.
 */
contract Idea is ERC20 {
	/* The CID of the Idea's metadata on IPFS, as defined in the V2 spec. */
	string public ipfsAddr;

	/* Existing votes delegated by a user for the purpose of resizing their
	 * vote upon a decrease in their balance of the Idea token. */
	mapping (address => CommitmentMap) commitments;

	/* Indicates that a proposal was attempted to be finalized, but failed to
	 * garner the necessary >50% majority. */
	event ProposalRejected(Proposal proposal);

	/* Indicates that a proposal was executed. */
	event ProposalAccepted(Proposal proposal, string oldPayload, string newPayload);

	/**
	 * Creates a new Idea DAO, with a corresponding ERC-20 token of name _name,
	 * symbol _symbol, and _supply tokens allocated to the sender of the
	 * message. Stores the given associated metadata in storage.
	 *
	 * @param _name - The name of the ERC20 governance token for the Idea
	 * @param _symbol - The currency symbol of the ERC20 governance token for
	 *  the Idea
	 * @param _supply - The number of tokens to create and allocate to the
	 *  sender of the message
	 * @param _ipfsAddr - The IPFS CID of metadata associated with the Idea to
	 *  store under the contract's ipfsAddr field
	 */
	constructor(string memory _name, string memory _symbol, uint256 _supply, string memory _ipfsAddr) ERC20(_name, _symbol) {
		ipfsAddr = _ipfsAddr;

		// Allocate specified tokens to author
		_mint(msg.sender, _supply);
	}

	/**
	 * Occurs on every transfer of a token. Detects any actives votes the user
	 * has registered that are now invalid because the user's balance
	 * decreased, or that can be pruned from storage because the voting period
	 * has passed.
	 */
	function _afterTokenTransfer(address from, address, uint256) internal override {
		if (commitments[from].nCommitments == 0)
			return;

		// Clear out any zombie commitments, and resolve any conflicts
		CommitmentMap storage commits = commitments[from];
		uint256 n = commits.nCommitments;

		for (uint256 i = 0; i < n;) {
			Proposal committee = commits.committed[i];
			Commitment storage commit = commits.commitments[committee];

			if (commit.weight == 0 || (block.timestamp > committee.expiry() && committee.expiry() != 0)) {
				// Zombie commitment. Clear out
				n--;

				commit.weight = 0;
				commits.committed[i] = commits.committed[n - 1];
				commits.committed.pop();

				commits.nCommitments--;
			} else if (commits.commitments[committee].weight > balanceOf(from)) {
				// The user's vote is no longer valid because their balance is less
				// than the number of votes they committed. Set the user's new
				// vote to the most they can pay
				commitVotes(committee, from, Commitment(committee, balanceOf(from), VoteKind.AFFIRMATIVE));
			}
		}
	}

	/**
	 * Registers a commitment for the Proposal at msg.sender that can be used
	 * to downsize the user's vote at a later point in time if necessary.
	 */
	function commitVotes(address voter, Commitment memory vote) public {
		commitVotes(Proposal(msg.sender), voter, vote);
	}

	/**
	 * Registers a commitment for the indicated Proposal.
	 */
	function commitVotes(Proposal proposal, address voter, Commitment memory vote) private {
		CommitmentMap storage existing = commitments[voter];

		if (existing.commitments[proposal].weight == 0) {
			existing.nCommitments++;
			existing.committed.push(proposal);
		}

		existing.commitments[proposal] = vote;
	}

	/**
	 * Gets the existing commitment of the voter to the proposal msg.sender.
	 */
	function commitment(address voter) public view returns (Commitment memory) {
		return commitments[voter].commitments[Proposal(msg.sender)];
	}

	/**
	 * Executes the proposal and returns true if the proposal was successful.
	 *
	 * Reverts if the proposal has not yet finished its voting period.
	 */
	function finalizeProposal(Proposal proposal) public returns (bool) {
		require(proposal.expiry() != 0 && block.timestamp >= proposal.expiry(),
				"Proposal voting period has not yet finished.");

		// Require a >50% majority to pass any proposal
		if (proposal.nAffirmative() * 100 / (totalSupply() * 100) < 50) {
			emit ProposalRejected(proposal);

			return false;
		}

		emit ProposalAccepted(proposal, ipfsAddr, proposal.payload());

		ipfsAddr = proposal.payload();

		return true;
	}
}
