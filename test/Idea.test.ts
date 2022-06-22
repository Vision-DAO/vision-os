import { ethers, waffle } from "hardhat";
import { expect } from "chai";
import { describe } from "mocha";

import IdeaContract from "../artifacts/contracts/Idea.sol/Idea.json";
import { Idea } from "../types/contracts/Idea";
import { IPFSClient, SCHEMA, MODULES, forAllModules, bytesEqual } from "./common";

import { create as createValidator } from "ipld-schema-validator";
import { parse as parseSchema, Schema } from "ipld-schema";
import { create } from "ipfs";
import fs from "fs";

const { loadFixture } = waffle;

/**
 * Deploys an instance of the beacon layer idea metadata schema with the given
 * details, returning the CID of the deployed instance.
 */
const createIdeaMetadata = async (ipfs: IPFSClient, title: string, description: string, payload: Uint8Array): Promise<string> => {
	return await ipfs.
};

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
	const fixture = async (): Promise<{ ipfs: IPFSClient, schema: Schema }> => {
		// Make sure not to save state between any IPFS instances by using
		// multiple, disjointed repos
		return { ipfs: await create({ repo: `ipfs_${Math.random() * 100}` }), schema: parseSchema(SCHEMA) };
	};

	describe("IdeaPayload schema", () => {
		it("Should be uploadable from a Uint8Array", async () => {
			const { ipfs, schema } = await loadFixture(fixture);
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
				const cid = await ipfs.dag.put(mod);
				const retrieved = await ipfs.dag.get(cid);

				expect(validator(retrieved)).to.be.true;
				expect(bytesEqual(mod, retrieved)).to.be.true;
			});
	});

	describe("IdeaMetadata schema", () => {
		it("Should be validly constructable from a JavaScript object", async () => {
			const { ipfs, schema } = await loadFixture(fixture);

			const validator = createValidator(schema, "IdeaMetadata");

			// Test making a metadata object for every available WASM module
			forAllModules(async (mod) => {
				const modCid = await ipfs.dag.put(mod);

				const exampleMetadata = {
					title: "Example DAO",
					description: "An example.",
					payload: modCid,
				};

				// Ensure that the schema does not conflict with our usage
				expect(validator(exampleMetadata)).to.be.true;
			});
		});

		it("Should be uploadable to the IPFS network", async () => {

		});
	});

	it("Should be constructable from an instance of the schema", async () => {
		const ipfs = await loadFixture(fixture);
	});
});
