import { ethers, waffle } from "hardhat";
import { expect, assert } from "chai";
import { describe } from "mocha";

import { IdeaMetadata, IdeaPayload } from "../types/schema";
import { IPFSClient, SCHEMA, forAllModules, bytesEqual, objectsEqual, TEST_DAO } from "./common";

import { parse as parseSchema, Schema } from "ipld-schema";
import { create } from "ipfs-core";
import { CID } from "multiformats/cid";

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
	const fixture = async (): Promise<{ address: string, ipfs: IPFSClient, schema: Schema }> => {
		// Make sure not to save state between any IPFS instances by using
		// multiple, disjointed repos
		return { address: (await ethers.getSigners())[0].address, ipfs: await create({ repo: `/var/tmp/ipfs_${Math.random() * 100}` }), schema: parseSchema(SCHEMA) };
	};

	describe("IdeaPayload schema", () => {
		it("Should be uploadable from a Uint8Array", async () => {
			const { ipfs, schema }: { ipfs: IPFSClient, schema: Schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaPayload");

			// Make sure each module can be treated as a schema instance
			forAllModules(async (mod) => {
				expect(validator(mod)).to.be.true;
				expect(await ipfs.dag.put(mod)).to.not.be.empty;
			});
		});

		it("Should be equivalent in locally stored and remote form", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaPayload");

			// Make sure the local disk and the remote version of the module are
			// exactly the same
			forAllModules(async (mod) => {
				const cid = await ipfs.dag.put(mod as IdeaPayload);
				const retrieved = await ipfs.dag.get(cid).then((res) => res.value) as IdeaPayload;

				expect(validator(retrieved)).to.be.true;
				expect(bytesEqual(mod, retrieved)).to.be.true;
			});
		});
	});

	describe("IdeaMetadata schema", () => {
		it("Should be validly constructable from a JavaScript object", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaMetadata");

			// Test making a metadata object for every available WASM module
			forAllModules(async (mod) => {
				const modCid = await ipfs.dag.put(mod);

				const exampleMetadata: IdeaMetadata = {
					title: TEST_DAO.title,
					description: TEST_DAO.description,
					payload: modCid,
				};

				// Ensure that the schema does not conflict with our usage
				expect(validator(exampleMetadata)).to.be.true;
				expect(await ipfs.dag.put(exampleMetadata)).to.not.be.empty;
			});
		});

		it("Should be uploadable to the IPFS network", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
			const validator = createValidator(schema, "IdeaMetadata");

			// Test making a metadata object for every available WASM module
			forAllModules(async (mod) => {
				const modCid = await ipfs.dag.put(mod);

				const exampleMetadata: IdeaMetadata = {
					title: TEST_DAO.title,
					description: TEST_DAO.description,
					payload: modCid,
				};

				const cid = await ipfs.dag.put(exampleMetadata);
				const retrieved = await ipfs.dag.get(cid).then((res) => res.value) as IdeaMetadata;

				expect(validator(retrieved)).to.be.true;
				expect(objectsEqual(exampleMetadata, retrieved)).to.be.true;
			});
		});
	});

	it("Should be constructable from an instance of the schema", async () => {
		const { address, ipfs, schema } = await loadFixture(fixture);

		// Try making an Idea for schema instances for every WASM module available
		// Verify that the state of the associated ERC-20 matches the parameters
		// we give it.
		forAllModules(async (mod) => {
			const modCid = await ipfs.dag.put(mod);
			const exampleMetadata: IdeaMetadata = {
				title: TEST_DAO.title,
				description: TEST_DAO.description,
				payload: modCid,
			};

			const cid = await ipfs.dag.put(exampleMetadata);

			// Deploy a DAO using the selected WASM module
			const Idea = await ethers.getContractFactory("Idea");
			const idea = await Idea.deploy(TEST_DAO.title, TEST_DAO.symbol, TEST_DAO.supply, cid.toString());

			expect(await idea.deployed()).to.not.be.empty;

			// Ensure successful deployment according to constructor parameters, and
			// necessary events were emitted
			const onChainMeta = await idea.ipfsAddr();

			expect(idea).to.contain("0x");
			assert(CID.isCID(onChainMeta), "Invalid CID");

			// Ensure that the Idea's payload and metadata is intact
			const metaVerifier = createValidator(schema, "IdeaMetadata");
			const metadata = await ipfs.dag.get(onChainMeta).then((res) => res.value) as IdeaMetadata;

			expect(metaVerifier(metadata)).to.be.true;
			expect(metadata.title).to.equal(TEST_DAO.title);
			expect(metadata.description).to.equal(TEST_DAO.description);

			const payloadVerifier = createValidator(schema, "IdeaPayload");
			const payload = await ipfs.dag.get(metadata.payload).then((res) => res.value);

			expect(payloadVerifier(payload)).to.be.true;
			expect(bytesEqual(payload, mod)).to.be.true;

			// Ensure that the details of the Idea's token are intact
			expect(await idea.balanceOf(address)).to.equal(TEST_DAO.supply);
			expect(await idea.symbol()).to.equal(TEST_DAO.symbol);
			expect(await idea.name()).to.equal(TEST_DAO.title);
		});
	});
});
