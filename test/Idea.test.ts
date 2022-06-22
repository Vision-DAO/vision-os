import { ethers, waffle } from "hardhat";
import chai from "chai";
import { describe } from "mocha";

import IdeaContract from "../artifacts/contracts/Idea.sol/Idea.json";
import { Idea } from "../types/contracts/Idea";

/**
 * Tests the functionality of the Idea constructor contract, its ERC20
 * properties, the IPLD schema, and not much else. Governance tests are in
 * Prop.test.ts.
 */
describe("Idea", () => {
	it("Should be constructable from an instance of the schema", async () => {

	});
});
