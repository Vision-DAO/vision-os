import "@typechain/hardhat";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-waffle";
import "hardhat-deploy";

import { HardhatUserConfig } from "hardhat/types";

const config: HardhatUserConfig = {
	solidity: "0.8.9",
	typechain: {
		outDir: "types",
	},
	networks: {
		mumbai: {
			chainId: 80001,
			url: "https://matic-mumbai.chainstacklabs.com",
			accounts: process.env.DEPLOYMENT_PRIVATE_KEY !== undefined ?
				[process.env.DEPLOYMENT_PRIVATE_KEY] : [],
		}
	}
};

export default config;
