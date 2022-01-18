const BigNumber = require("bignumber.js");

/* Attributes of the root node in the idea tree used for deployment */
module.exports = {
  name: "CHID DAO",
  ticker: "CHID",

  // One per student, with 18 decimals of precision
  shares: new BigNumber("13e18"),

  // Hard-coded address of the details of this idea on IPFS (an HTML file)
  // containing an overview of the concept (static/index.html)
  detailsIpfsID: "QmWd94nKbgZHn9CjvDCmSJfUXFdcvScfC87xVGP6Lc7DzG",
};

