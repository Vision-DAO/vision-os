const Dao = artifacts.require("Idea");
const Prop = artifacts.require("Prop");
const ROOT_IDEA = require("./conf");

module.exports = function (deployer) {
	deployer.deploy(Dao, ROOT_IDEA.name, ROOT_IDEA.ticker, ROOT_IDEA.shares, ROOT_IDEA.detailsIpfsID);
	deployer.deploy(Prop, ...ROOT_IDEA.propArgs);
};
