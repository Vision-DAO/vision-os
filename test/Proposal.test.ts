import { ethers, waffle, network } from "hardhat";
import { expect, assert } from "chai";
import { describe } from "mocha";

import { IdeaMetadata, ProposalPayload, ProposalMetadata } from "../types/schema";
import { IPFSClient, TEST_DAO, fixture, forAllModuleCIDs, TEST_PROP, propFixture, governorFixture, forAllModuleFixtures } from "./common";

import { Schema } from "ipld-schema";
import { CID } from "multiformats/cid";

const { create: createValidator } = require("ipld-schema-validator");
const { loadFixture } = waffle;

/**
 * Tests the functionality of the Proposal contract, schema, and the governance
 * functions of the Idea contract.
 */
describe("Proposal", () => {
	describe("ProposalPayload schema", () => {
		it("Should be uploadable from an IdeaMetadata instance", async () => {
			const { ipfs, schema }: { ipfs: IPFSClient, schema: Schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "ProposalPayload");

			// Make sure each module can be treated as a schema instance by uploading
			// an instance, and treating its link as a ProposalPayload
			await forAllModuleCIDs(ipfs, async (modCid) => {
				const payload: IdeaMetadata = {
					title: TEST_DAO.title,
					description: TEST_DAO.description,
					payload: [modCid],
				};

				const cid = await ipfs.dag.put(payload);

				expect(cid).to.not.be.empty;
				expect(validator(cid)).to.be.true;
			});
		});

		it("Should be equivalent in locally stored and remote form", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "ProposalPayload");

			// Make sure the local disk and the remote version of the module are
			// exactly the same
			await forAllModuleCIDs(ipfs, async (modCid) => {
				const payload: IdeaMetadata = {
					title: TEST_DAO.title,
					description: TEST_DAO.description,
					payload: [modCid],
				};

				const cid: ProposalPayload = await ipfs.dag.put(payload);
				const retrieved = (await ipfs.dag.get(cid)).value as IdeaMetadata;

				expect(validator(cid)).to.be.true;
				expect(payload).to.eql(retrieved);
			});
		});
	});

	describe("ProposalMetadata schema", () => {
		it("Should be validly constructable from a JavaScript object", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "ProposalMetadata");

			// Make a proposal from the test data in common
			expect(validator(TEST_PROP.meta)).to.be.true;
			expect(await ipfs.dag.put(TEST_PROP.meta)).to.not.be.empty;
		});

		it("Should be uploadable to the IPFS network", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "ProposalMetadata");

			// Test making and deserializing metadata
			const cid = await ipfs.dag.put(TEST_PROP.meta);
			const retrieved = (await ipfs.dag.get(cid)).value as ProposalMetadata;

			expect(validator(retrieved)).to.be.true;
			expect(TEST_PROP.meta).to.eql(retrieved);
		});
	});

	it("Should be constructable from an instance of the schema", async () => {
		const { ipfs, schema } = await loadFixture(fixture);

		await forAllModuleCIDs(ipfs, async (modCid) => {
			// Deploy a DAO using basic metadata, and without any payloads
			const idea = await governorFixture(ipfs);

			// What the DAO's metadata will be changed to
			const newPayload: IdeaMetadata = {
				title: TEST_DAO.altTitle,
				description: TEST_DAO.altDescription,
				payload:  [modCid],
			};

			// Deploy a proposal to change the DAO's payload to something new
			const { prop, payloadCid } = await propFixture(idea, newPayload, ipfs);
			expect(await prop.deployed()).to.not.be.empty;

			// Ensure successful deployment according to constructor parameters, and
			// necessary events were emitted
			const onChainMeta = CID.parse(await prop.ipfsAddr());

			expect(prop.address).to.contain("0x");
			assert(onChainMeta !== null, "Invalid CID");

			// Ensure that the Proposal's payload and metadata is intact
			const metaVerifier = createValidator(schema, "ProposalMetadata");
			const metadata = (await ipfs.dag.get(onChainMeta)).value as ProposalMetadata;

			expect(metaVerifier(metadata)).to.be.true;
			expect(metadata).to.eql(TEST_PROP.meta);

			// The proposal should have the CID of new metadata for the DAO
			const payloadRefVerifier = createValidator(schema, "ProposalPayload");
			const payloadRef = CID.parse(await prop.payload());

			assert(payloadRef !== null, "Invalid CID");

			expect(payloadRefVerifier(payloadRef)).to.be.true;
			expect(payloadRef).to.eql(payloadCid);

			// The metadata located at the payload reference should be the same as
			// was constructed above
			expect((await ipfs.dag.get(payloadRef)).value).to.eql(newPayload);

			// Ensure that the details of the Proposal are intact
			expect(await prop.governor()).to.equal(idea.address);
			expect(await prop.active()).to.be.false;
			expect(await prop.duration()).to.equal(TEST_PROP.duration);

			// State should be uninitialized
			expect(await prop.expiry()).to.equal(0);
			expect(await prop.nVotes()).to.equal(0);
			expect(await prop.nVoters()).to.equal(0);
			expect(await prop.nAffirmative()).to.equal(0);
		});
	});

	it("Should be initiated only by its owner", async () => {
		await forAllModuleFixtures(async ({ prop }) => {
			// The first signer should be able to initiate it, but only the
			// first signer, and initiateVotingPeriod should not be reentrant
			const signers = await ethers.getSigners();
			await expect(prop.connect(signers[1]).initiateVotingPeriod()).to.be.reverted;

			const initTx = prop.initiateVotingPeriod();
			await expect(initTx)
				.to.emit(prop, "VoteStarted")
				.withArgs(signers[0].address);

			const receipt = await initTx;
			assert(receipt.blockNumber !== undefined, "Deployment tx not mined.");

			// The voting period should be over duration seconds after the init tx
			expect(await prop.expiry())
				.to.equal(
					(await ethers.provider.getBlock(receipt.blockNumber).then((block) => block.timestamp))
					+ await prop.duration().then((dur) => dur.toNumber()))
			expect(await prop.active()).to.be.true;
			await expect(prop.initiateVotingPeriod()).to.be.reverted;
		});
	});

	it("Should be votable", async () => {
		await forAllModuleFixtures(async ({ idea, prop }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			// Make an affirmative vote to the Proposal, which should increment
			// the nAffirmative by the number of votes I specify, nVoters by 1,
			// and nVotes overall by the number of votes
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply);

			// State should be updated accordingly
			expect(await prop.nVoters()).to.equal(1);
			expect(await prop.nAffirmative()).to.equal(TEST_DAO.supply);
			expect(await prop.nVotes()).to.equal(TEST_DAO.supply);

			// The parent contract's commitments should be updated
			const commit = await idea.commitment(prop.address, signers[0].address);
			expect(commit.nature).to.equal(0);
			expect(commit.weight).to.equal(TEST_DAO.supply);
			expect(commit.dependent).to.equal(prop.address);
		});
	});

	// The user should be able to send tokens freely, decreasing their
	// vote weight accordingly
	it("Should resize a user's vote upon losing balance", async () => {
		await forAllModuleFixtures(async ({ idea, prop }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply);

			// The user's vote size should be subtracted by 1
			await expect(idea.transfer(signers[1].address, 1))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply.sub(1));

			expect(await prop.nAffirmative()).to.equal(TEST_DAO.supply.sub(1));
			expect(await prop.nVotes()).to.equal(TEST_DAO.supply.sub(1));
			expect(await prop.nVoters()).to.equal(1);
		});
	});

	// Try to finish the prop before the expiration
	it("Should not be finalizble before the voting block expires", async () => {
		await forAllModuleFixtures(async ({ prop, idea }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();

			// The user should not be able to finalize a proposal before the expiry
			// date
			await expect(idea.finalizeProposal(prop.address))
				.to.be.reverted;
		});
	});

	// Advance time to after the expiration, and finalize the proposal
	it("Should be finalizable", async () => {
		await forAllModuleFixtures(async ({ idea, prop, payloadCid }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			// Make an affirmative vote to the Proposal, which should increment
			// the nAffirmative by the number of votes I specify, nVoters by 1,
			// and nVotes overall by the number of votes
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply);

			const oldPayloadCid = await idea.ipfsAddr();
			await network.provider.request({ method: "evm_setNextBlockTimestamp", params: [(await prop.expiry()).toNumber() + 1] });

			// The Idea's metadata should change on successful finalization,
			// but the finalize method should not be reentrant
			await expect(idea.finalizeProposal(prop.address))
				.to.emit(idea, "ProposalAccepted")
				.withArgs(prop.address, oldPayloadCid, payloadCid.toString());
			expect(await idea.ipfsAddr())
				.to.eql(payloadCid.toString());
			await expect(idea.finalizeProposal(prop.address))
				.to.be.reverted;
		});
	});

	it("Should reset a user's commitment upon finalizing", async () => {
		await forAllModuleFixtures(async ({ prop, payloadCid, idea }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			// Make an affirmative vote to the Proposal, which should increment
			// the nAffirmative by the number of votes I specify, nVoters by 1,
			// and nVotes overall by the number of votes
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply);

			const oldPayloadCid = await idea.ipfsAddr();
			await network.provider.request({ method: "evm_setNextBlockTimestamp", params: [(await prop.expiry()).toNumber() + 1] });

			// The Idea's metadata should change on successful finalization,
			// but the finalize method should not be reentrant
			await expect(idea.finalizeProposal(prop.address))
				.to.emit(idea, "ProposalAccepted")
				.withArgs(prop.address, oldPayloadCid, payloadCid.toString());

			// The user's commitments should be wiped upon the next transaction
			await idea.transfer(signers[1].address, 1);

			const commit = await idea.commitment(prop.address, signers[0].address);
			expect(commit.weight).to.equal(0);
		});
	});

	// The user should no longer be able to vote after the proposal is
	// finished
	it("Should not be votable after the expiration timestamp", async () => {
		await forAllModuleFixtures(async ({ prop }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			// Make an affirmative vote to the Proposal, which should increment
			// the nAffirmative by the number of votes I specify, nVoters by 1,
			// and nVotes overall by the number of votes
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply);

			await network.provider.request({ method: "evm_setNextBlockTimestamp", params: [(await prop.expiry()).toNumber() + 1] });

			// Any attempt to vote should be reverted after the expiry
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply.sub(1)))
				.to.be.reverted;
		});
	});

	it("Should not pass without more than a 50% majority", async () => {
		await forAllModuleFixtures(async ({ idea, prop }) => {
			// Start the proposal
			await prop.initiateVotingPeriod();
			const signers = await ethers.getSigners();

			// Make an affirmative vote to the Proposal, which should increment
			// the nAffirmative by the number of votes I specify, nVoters by 1,
			// and nVotes overall by the number of votes
			await expect(prop["castVote(uint8,uint256)"](0, TEST_DAO.supply.div(2)))
				.to.emit(prop, "VoteCast")
				.withArgs(signers[0].address, 0, TEST_DAO.supply.div(2));

			const oldPayloadCid = await idea.ipfsAddr();
			await network.provider.request({ method: "evm_setNextBlockTimestamp", params: [(await prop.expiry()).toNumber() + 1] });

			// The Idea's metadata should not change, since the proposal didn't get
			// > 50% of the vote
			await expect(idea.finalizeProposal(prop.address))
				.to.emit(idea, "ProposalRejected")
				.withArgs(prop.address);
			expect(await idea.ipfsAddr())
				.to.eql(oldPayloadCid);
			await expect(idea.finalizeProposal(prop.address))
				.to.be.reverted;

			// The user's commitments should be wiped upon the next transaction
			await idea.transfer(signers[1].address, 1);

			const commit = await idea.commitment(prop.address, signers[0].address);
			expect(commit.weight).to.equal(0);
		});
	});
});
