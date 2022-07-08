import { ethers, waffle } from "hardhat";
import { expect, assert } from "chai";
import { describe } from "mocha";

import { IdeaMetadata, IdeaPayload } from "../types/schema";
import { IPFSClient, forAllModules, forAllModuleCIDs, TEST_DAO, fixture, MODULES } from "./common";

import { Schema } from "ipld-schema";
import { CID } from "multiformats/cid";
import fs from "fs";

const { create: createValidator } = require("ipld-schema-validator");
const { loadFixture } = waffle;

/**
 * Tests the functionality of the Idea constructor contract, its ERC20
 * properties, the IPLD schema, and not much else. Governance tests are in
 * Prop.test.ts.
 *
 * HELP: These tests rely on two packages made by protocol labs that we use for
 * validating that data with a shape our solidity and typescript code expects
 * conforms with the schema declared in the fixtures folder. You can see some
 * helpful examples and documentation
 * here: https://github.com/ipld/js-ipld-schema
 */
describe("Idea", () => {
	describe("IdeaPayload schema", () => {
		it("Should be uploadable from a Uint8Array", async () => {
			const { ipfs, schema }: { ipfs: IPFSClient, schema: Schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaPayload");

			// Make sure each module can be treated as a schema instance
			await forAllModules(async (mod) => {
				expect(validator(mod)).to.be.true;
				expect(await ipfs.dag.put(mod)).to.not.be.empty;
			});
		});

		it("Should be equivalent in locally stored and remote form", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaPayload");

			// Make sure the local disk and the remote version of the module are
			// exactly the same
			await forAllModules(async (mod) => {
				const cid = await ipfs.dag.put(mod as IdeaPayload);
				const retrieved = await ipfs.dag.get(cid).then((res) => res.value) as IdeaPayload;

				expect(validator(retrieved)).to.be.true;
				expect(mod).to.eql(retrieved);
			});
		});
	});

	describe("IdeaMetadata schema", () => {
		it("Should be validly constructable from a JavaScript object", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaMetadata");

			// Test making a metadata object for every available WASM module

			// Payloads have the WASM modules themselves, and the JS used to attach
			// them
			const combinedPayload: IdeaPayload[] = await Promise.all(MODULES.map(([mod, loader]) => { return {
					loader: fs.readFileSync(loader).toString(),
					module: fs.readFileSync(mod),
				};
			}));

			// Payloads are not written in-line, they are stored via CID references
			const payloadCIDs: CID[] = await Promise.all(combinedPayload.map((payload) => ipfs.dag.put(payload)));

			const exampleMetadata: IdeaMetadata = {
				title: TEST_DAO.title,
				payload: payloadCIDs,
				description: TEST_DAO.description,
			};

			// Ensure that the schema does not conflict with our usage
			expect(validator(exampleMetadata)).to.be.true;
			expect(await ipfs.dag.put(exampleMetadata)).to.not.be.empty;
		});

		it("Should be uploadable to the IPFS network", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaMetadata");

			// Test making a metadata object and retrieving it from IPFS
			const combinedPayload: IdeaPayload[] = await Promise.all(MODULES.map(([mod, loader]) => { return {
					loader: fs.readFileSync(loader).toString(),
					module: fs.readFileSync(mod),
				};
			}));

			const payloadCIDs: CID[] = await Promise.all(combinedPayload.map((payload) => ipfs.dag.put(payload)));
			const exampleMetadata: IdeaMetadata = {
				title: TEST_DAO.title,
				payload: payloadCIDs,
				description: TEST_DAO.description,
			};

			const cid = await ipfs.dag.put(exampleMetadata);
			const retrieved = await ipfs.dag.get(cid).then((res) => res.value) as IdeaMetadata;

			expect(validator(retrieved)).to.be.true;
			expect(exampleMetadata).to.eql(retrieved);
		});
	});

	it("Should be constructable from an instance of the schema", async () => {
		const { address, ipfs, schema } = await loadFixture(fixture);

		// Try making an Idea for schema instances for every WASM module available
		// Verify that the state of the associated ERC-20 matches the parameters
		// we give it.
		await forAllModules(async (mod) => {
			const modCid = await ipfs.dag.put(mod);
			const exampleMetadata: IdeaMetadata = {
				title: TEST_DAO.title,
				description: TEST_DAO.description,
				payload: [modCid],
			};

			const cid = await ipfs.dag.put(exampleMetadata);

			// Deploy a DAO using the selected WASM module
			const Idea = await ethers.getContractFactory("Idea");
			const idea = await Idea.deploy(TEST_DAO.title, TEST_DAO.symbol, TEST_DAO.supply, cid.toString());

			expect(await idea.deployed()).to.not.be.empty;

			// Ensure successful deployment according to constructor parameters, and
			// necessary events were emitted
			const onChainMeta = CID.parse(await idea.ipfsAddr());

			expect(idea.address).to.contain("0x");
			assert(onChainMeta !== null, "Invalid CID");

			// Ensure that the Idea's payload and metadata is intact
			const metaVerifier = createValidator(schema, "IdeaMetadata");
			const metadata = await ipfs.dag.get(onChainMeta).then((res) => res.value) as IdeaMetadata;

			expect(metaVerifier(metadata)).to.be.true;
			expect(metadata.title).to.equal(TEST_DAO.title);
			expect(metadata.description).to.equal(TEST_DAO.description);

			const payloadVerifier = createValidator(schema, "IdeaPayload");
			const payload = await ipfs.dag.get(metadata.payload[0]).then((res) => res.value);

			expect(payloadVerifier(payload)).to.be.true;
			expect(payload).to.eql(mod);

			// Ensure that the details of the Idea's token are intact
			expect(await idea.balanceOf(address)).to.equal(TEST_DAO.supply);
			expect(await idea.symbol()).to.equal(TEST_DAO.symbol);
			expect(await idea.name()).to.equal(TEST_DAO.title);
		});
	});

	// Idea is an ERC-20, so it should be transferrable between accounts
	it("Should be transferrable", async () => {
		const { ipfs } = await loadFixture(fixture);
		const [sender, recipient,] = await ethers.getSigners();

		// Try:
		// - A valid transaction
		// - An invalid transaction
		forAllModuleCIDs(ipfs, async (modCid) => {
			const meta: IdeaMetadata = {
				title: TEST_DAO.title,
				description: TEST_DAO.description,
				payload: [modCid],
			};

			const cid = await ipfs.dag.put(meta);

			const Idea = await ethers.getContractFactory("Idea");
			const idea = await Idea.deploy(TEST_DAO.title, TEST_DAO.symbol, TEST_DAO.supply, cid.toString());
			await idea.deployed();

			// The user should be able to make the transaction once if they spend
			// all their balance
			await expect(idea.transfer(recipient.address, TEST_DAO.supply))
				.to.emit(idea, "Transfer");
			await expect(idea.transfer(recipient.address, TEST_DAO.supply))
				.to.be.reverted;

			// The user's new balance should be zero, and the recipient's balance
			// should be the supply
			expect(await idea.balanceOf(sender.address)).to.equal(0);
			expect(await idea.balanceOf(recipient.address)).to.equal(TEST_DAO.supply);
		});
	});
});
