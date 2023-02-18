import { IdeaMetadata } from "../types/schema";
import { create } from "ipfs-core";
import { BigNumber } from "ethers";
import { DeployFunction } from "hardhat-deploy/types";

import "hardhat-deploy";
import fs from "fs";

/**
 * Production-ish details of the DAO to be deployed.
 */
const DEP_DAO_DETAILS: IdeaMetadata = {
	title: "Vision DAO",
	description: fs.readFileSync("fixtures/dao.md").toString(),
	payload: [],
};

/**
 * Tokens details of the DAO.
 */
const DEP_DAO_CONF = {
	symbol: "VIS",

	// Release 1 million tokens with 18 units of precision
	supply: BigNumber.from(1000000).mul(BigNumber.from(10).pow(18)),
};

/**
 * Deploys a DAO, delegating 1,000,000 VIS to the specified DAO_SETUP_ADDRESS.
 */
const deployment: DeployFunction = async ({
	deployments: { deploy },
	getUnnamedAccounts,
}) => {
	const deployer = await getUnnamedAccounts().then((accounts) => accounts[0]);

	// Upload the metadata for the DAO
	const ipfs = await create();
	const cid = await ipfs.dag.put(DEP_DAO_DETAILS);

	// Create the DAO
	await deploy("Idea", {
		from: deployer,
		args: [
			DEP_DAO_DETAILS.title,
			DEP_DAO_CONF.symbol,
			DEP_DAO_CONF.supply,
			cid.toString(),
		],
	});
};

export default deployment;
