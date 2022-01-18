const Dao = artifacts.require("Idea");
const ROOT_IDEA = require("./conf");

module.exports = function (deployer) {
  deployer.deploy(Dao, ROOT_IDEA.name, ROOT_IDEA.ticker, ROOT_IDEA.shares, ROOT_IDEA.detailsIpfsID);
};
