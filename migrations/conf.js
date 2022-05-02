const BigNumber = require("bignumber.js");

/* Attributes of the root node in the idea tree used for deployment */
module.exports = {
	name: "Test CHID DAO",
	ticker: "TEST",

	// One per student, with 18 decimals of precision
	shares: new BigNumber("13e18"),

	// Hard-coded address of the details of this idea on IPFS (an HTML file)
	// containing an overview of the concept (static/index.html)
	detailsIpfsID: "QmWd94nKbgZHn9CjvDCmSJfUXFdcvScfC87xVGP6Lc7DzG",

	// Details for a testing proposal (used for validating contract ABI's)
	propArgs: ["Test proposal", "0x928613da9dE038458c29fe34066fbbDe74A2DB9f", "0x928613da9dE038458c29fe34066fbbDe74A2DB9f", "0x928613da9dE038458c29fe34066fbbDe74A2DB9f", 0, "", 1],
};

