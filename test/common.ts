import { IdeaPayload, IdeaMetadata } from "../types/schema";
import { Idea } from "../types/contracts/Idea"
import { Proposal } from "../types/contracts/Prop.sol/Proposal";
import { ethers, waffle } from "hardhat";
import { parse as parseSchema, Schema } from "ipld-schema";
import { CID } from "multiformats/cid";
import { ProposalMetadata } from "../types/schema";
import { BigNumber } from "ethers";

import fs from "fs";
import path from "path";
import { create } from "ipfs-core";

const { loadFixture } = waffle;

/**
 * An instance of IPFS used for harnessing a test.
 */
export type IPFSClient = Awaited<ReturnType<typeof create>>;

/**
 * The title, description, symbol, and supply of an example DAO.
 */
export const TEST_DAO = {
	title: "Example DAO",
	altTitle: "New Example DAO",

	description: "An example.",
	altDescription: "Edited.",

	symbol: "TEST",
	supply: BigNumber.from(21000000).mul(BigNumber.from(10).pow(18)),
};

/**
 * The title and description of a sample proposal.
 */
export const TEST_PROP = {
	meta: {
		title: "Example Proposal",
		description: "An example.",
	} as ProposalMetadata,
	// Voting period that only lasts 1 minute
	duration: 60,
};

/**
 * The path to the file containing an IPLD schema to parse describing Vision
 * metadata.
 */
export const SCHEMA_PATH: string = path.resolve(process.env.SCHEMA_PATH || "fixtures/beacon_layer.ipldsch");

/**
 * The path to a directory of .wasm files that should be loaded for testing
 * purposes. No extensive testing of these modules' functionality is performed.
 */
export const MODULES_PATH: string = path.resolve(process.env.MODULES_PATH || "fixtures/modules/target/wasm32-unknown-unknown/release/");

/**
 * wasm-pack will stubbornly compile the root src/lib.rs for the modules
 * fixture, but we don't want to actually run it.
 */
export const MODULE_IGNORES: string[] = (process.env.MODULE_IGNORES || "beacon_dao_modules").split(" ");

/**
 * The paths to every WASM module that can be used by the testing suite.
 */
export const MODULES: string[] = fs
	.readdirSync(MODULES_PATH)
	.filter((path) =>
		!(path.substring(path.lastIndexOf(".")) in MODULE_IGNORES))
	.filter((path) => path.includes(".wasm"))
	.map((p) => path.resolve(MODULES_PATH, p)
);

/**
 * The IPLD schema used for validating compatibility with test methodology.
 */
export const SCHEMA: string = fs.readFileSync(SCHEMA_PATH, "utf-8");

/**
 * Executes a test for every WASM module loaded for testing.
 */
export const forAllModules = async (fn: (module: IdeaPayload) => Promise<void>) => {
	for (const modPath of MODULES) {
		const modContents = fs.readFileSync(modPath);

		await fn(modContents);
	}
}

/**
 * Executes a test for every WASM module uploaded for testing.
 */
export const forAllModuleCIDs = async (ipfs: IPFSClient, fn: (module: CID) => Promise<void>) => {
	for (const modPath of MODULES) {
		const modContents = fs.readFileSync(modPath);

		await fn(await ipfs.dag.put(modContents));
	}
}

/**
 * Executes a test with an injected IPFS, schema, module CID, and default
 * metadata for every loaded module.
 */
export const forAllModuleFixtures = async (fn: (env: { ipfs: IPFSClient, schema: Schema, payload: IdeaMetadata, idea: Idea, prop: Proposal, payloadCid: CID }) => Promise<void>) => {
	const { ipfs, schema } = await loadFixture(fixture);

	await forAllModuleCIDs(ipfs, async (modCid) => {
		const payload: IdeaMetadata = {
			title: TEST_DAO.title,
			description: TEST_DAO.description,
			payload: [modCid],
		};

		const idea = await governorFixture(ipfs);
		// What the DAO's metadata will be changed to
		const newPayload: IdeaMetadata = {
			title: TEST_DAO.altTitle,
			description: TEST_DAO.altDescription,
			payload:  [modCid],
		};

		const { prop, payloadCid } = await propFixture(idea, newPayload, ipfs);

		await fn({ ipfs, schema, payload, idea, prop, payloadCid });
	})
};

/**
 * Utility function used by hardhat to load common trappings of the testing
 * harness.
 */
export const fixture = async (): Promise<{ address: string, ipfs: IPFSClient, schema: Schema }> => {
	// Make sure not to save state between any IPFS instances by using
	// multiple, disjointed repos
	return { address: (await ethers.getSigners())[0].address, ipfs: await create({ repo: `/var/tmp/ipfs_${Math.random() * 100}` }), schema: parseSchema(SCHEMA) };
};

/**
 * Deploys an instance of a parent governing contract that can be used for
 * testing proposals.
 */
export const governorFixture = async (ipfs: IPFSClient): Promise<Idea> => {
	const originalPayload = await ipfs.dag.put({ title: TEST_DAO.title, description: TEST_DAO.description, payload: [] } as IdeaMetadata)
	const Idea = await ethers.getContractFactory("Idea");
	const idea = await Idea.deploy(TEST_DAO.title, TEST_DAO.symbol, TEST_DAO.supply, originalPayload.toString());

	return await idea.deployed();
};

/**
 * Deploys a proposal owned by the first signer, and governed by the
 * given contract.
 */
export const propFixture = async (gov: Idea, payload: IdeaMetadata, ipfs: IPFSClient): Promise<{ prop: Proposal, payloadCid: CID }> => {
	// Deploy a new payload for the Proposal changing the title, description,
	// and binary payload
	const payloadCid = await ipfs.dag.put(payload)

	// Deploy metadata for the Proposal
	const cid = await ipfs.dag.put(TEST_PROP.meta);

	// Deploy a proposal to change the DAO's payload to something new
	const Proposal = await ethers.getContractFactory("Proposal");
	const prop = await Proposal.deploy(cid.toString(), payloadCid.toString(), TEST_PROP.duration, gov.address);
	await prop.deployed();

	return { prop, payloadCid };
};
