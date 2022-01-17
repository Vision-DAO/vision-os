const Dao = artifacts.require("Idea");

/* Attributes of the root node in the idea tree used for deployment */
const ROOT_IDEA = {
  name: "CHID DAO",
  ticker: "DAO",

  // One per student, with 18 decimals of precision
  shares: 13,

  // Hard-coded address of the details of this idea on IPFS (an HTML file)
  // containing an overview of the concept (static/index.html)
  detailsIpfsID: "QmWd94nKbgZHn9CjvDCmSJfUXFdcvScfC87xVGP6Lc7DzG",
};

module.exports = function (deployer) {
  deployer.deploy(Dao, ROOT_IDEA.name, ROOT_IDEA.ticker, ROOT_IDEA.shares, ROOT_IDEA.detailsIpfsID);
};
