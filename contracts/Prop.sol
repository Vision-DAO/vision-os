// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "../node_modules/@openzeppelin/contracts/access/Ownable.sol";
import "./Idea.sol";
import "hardhat/console.sol";

/**
 * An iterable set of all the commitments made by the user to proposals for
 * a governing contract.
 */
struct CommitmentMap {
	/* The proposals the user has committed to, and the details of their
	 * commitment. */
	mapping (Proposal => Commitment) commitments;
	Proposal[] committed;

	/* The number of Proposals the user committed to */
	uint256 nCommitments;
}

/**
 * A struct describing a user's previously allocated vote, for the purpose of
 * dynamically downsizing their vote if their balance decreases.
 */
struct Commitment {
	Proposal dependent;
	uint256 weight;
	VoteKind nature;
}

/**
 * The nature of a vote for a proposal. Either for, or against a proposal.
 */
enum VoteKind {
	AFFIRMATIVE,
	NEGATIVE
}

/**
 * A contract that implements a metadata-associated proposal to alter a parent
 * DAO's associated metadata in a certain way, as defined in the Vision V2
 * spec.
 */
contract Proposal is Ownable {
	/* The CID of metadata associated with the Proposal describing its
	 * contents, and the execution payload of the proposal. */
	string public ipfsAddr;
	string public payload;

	/* The number of yes votes, total votes, and total voting addresses. */
	uint256 public nAffirmative;
	uint256 public nVoters;
	uint256 public nVotes;

	/* The length of the proposal's voting period, the timestamp after
	 * which votes will no longer be accepted, and the timestamp at which
	 * the proposal was finalized. */
	uint256 public duration;
	uint256 public expiry;
	uint256 public closedAt;

	/* The Idea whose tokens represent votes for this proposal. */
	Idea public governor;

	/* Indicates that a user successfully cast a vote in a given direction,
	 * and with the indicated magnitude. */
	event VoteCast(address voter, VoteKind nature, uint256 weight);

	/* Indicates that an organizer of the vote initiated the voting period. */
	event VoteStarted(address organizer);

	/**
	 * Access control allowing a method to only be called if voting has not
	 * begun.
	 */
	modifier notBegun {
		require(!active(), "The voting period has already begun.");
		_;
	}

	/**
	 * Access control allowing a method to only be called if voting has begun,
	 * and has not yet finished.
	 */
	modifier isActive {
		require(active(), "The voting period has not yet begun.");
		_;
	}

	modifier isGovernor {
		require(msg.sender == address(governor),
				"The caller must be the governor of the proposal.");
		_;
	}

	/**
	 * Creates a new proposal with the given metadata, voting period length,
	 * and parent contract. The parent contract must be an instance of the Idea
	 * contract.
	 *
	 * @param _ipfsAddr - The CID of associated metadata accessible via the
	 *  ipfsAddr method describing the contents of the proposal
	 * @param _payload - The CID of new metadata to associate with the
	 *  governing contract upon successful execution
	 * @param _duration - The number of seconds that the voting period will
	 *  last, after it has begun
	 * @param _governor - The address of the contract whose tokens represent
	 *  votes in either direction for the proposal.
	 */
	constructor(string memory _ipfsAddr, string memory _payload, uint256 _duration, Idea _governor) {
		ipfsAddr = _ipfsAddr;
		payload = _payload;

		// Expiry determined after the vote is initiated
		duration = _duration;
		expiry = 0;
		closedAt = 0;

		governor = _governor;
	}

	/**
	 * Returns true if the voting period has been initiated, but has not yet
	 * been completed.
	 */
	function active() public view returns (bool) {
		return expiry != 0 && block.timestamp < expiry;
	}

	/**
	 * Attempts to initiate the voting period for the proposal.
	 * Reverts if the voting period has already begun.
	 * Can only be called by the author of the proposal.
	 */
	function initiateVotingPeriod() public onlyOwner notBegun {
		require(expiry == 0, "Voting period already started.");

		expiry = block.timestamp + duration;
		emit VoteStarted(msg.sender);
	}

	/**
	 * Marks the proposal as spent, meaning it can no longer be finalized.
	 */
	function closeProposal() public isGovernor {
		closedAt = block.timestamp;
	}

	/**
	 * Casts a vote in the affirmative or negative for a voter identified by an
	 * address.
	 */
	function castVote(address voter, VoteKind nature, uint256 weight) public isGovernor {
		delegateVote(voter, nature, weight);
	}

	function delegateVote(address voter, VoteKind nature, uint256 weight) private {
		Commitment memory prevVote = governor.commitment(this, voter);

		// Increment the vote count if the user hadn't voted previously.
		// Set the new total number of votes to the total number of votes
		// without the user's old vote, but with their new vote
		nVoters += prevVote.weight == 0 ? 1 : 0;
		nVotes = nVotes - prevVote.weight + weight;

		// Subtract the user's old affirmative votes if there were any
		if (nature == VoteKind.AFFIRMATIVE) {
			uint256 newAffirmative = nAffirmative + weight;

			if (prevVote.nature == VoteKind.AFFIRMATIVE)
				newAffirmative -= prevVote.weight;

			nAffirmative = newAffirmative;
		}

		governor.commitVotes(voter, Commitment(this, weight, nature));
		emit VoteCast(voter, nature, weight);
	}

	/**
	 * Casts a vote in the affirmative or negative, and with the given number
	 * of votes, weight.
	 *
	 * Reverts if the user's balance is less than the specified weight.
	 * Reverts if the voting period is not active.
	 */
	function castVote(VoteKind nature, uint256 weight) public isActive {
		require(governor.balanceOf(msg.sender) >= weight,
				"Insufficient balance to cast specified number of votes.");

		delegateVote(msg.sender, nature, weight);
	}
}
