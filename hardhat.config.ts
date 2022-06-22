import "@typechain/hardhat";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-waffle";

import { HardhatUserConfig } from "hardhat/types";

const config: HardhatUserConfig = {
	solidity: "0.8.9",
	typechain: {
		outDir: "types",
	},
};

export default config;
