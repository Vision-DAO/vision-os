const Idea = artifacts.require("Idea");
const Prop = artifacts.require("Prop");
const ROOT_IDEA = require("../migrations/conf");
const BigNumber = require("bignumber.js");
const { time } = require("@openzeppelin/test-helpers");

// Creates a new DAO, effectively resetting state for the test.
const withBlankDAO = () => Idea.new(ROOT_IDEA.name, ROOT_IDEA.ticker, ROOT_IDEA.shares, ROOT_IDEA.detailsIpfsID);

// Deploys a test idea contract, returning the contract and the specification
// for the contract
const withTestIdea = async () => {
  const token = {
    name: "new idea",
    ticker: "NEW",
    shares: new BigNumber("1e21"),
    detailsIpfsID: "",
  };

  const idea = await Idea.new(token.name, token.ticker, token.shares, token.detailsIpfsID);

  return [token, idea];
};

const withProp = (juris, idea) => Prop.new(juris.address, idea.address, juris.address, 1, "", 1);

// All nodes in the tree are represented by instances of the Idea contract, and
// edges are formed by funding schedules
contract("Idea", async accounts => {
  // Checks that the entire allocated supply of the given token was allocated
  // in entirety to the first account available
  const checkSupply = async (idea) => {
    // The first account should have all of the tokens allocated by the idea
    const balance = await idea.balanceOf.call(accounts[0]);
    const totalSupply = await idea.totalSupply.call();

    // TODO: See if there's another way to assert equality for bignumbers
    // because I don't like this lol
    assert.equal(totalSupply.toString(), balance.toString(), "Initial account wasn't allocated entire token supply.");
  };

  // Ensures that the metadata of the given ERC20 is in line with thsoe in the
  // given JS object
  const checkMetadata = async (idea, spec) => {
    assert.equal((await idea.totalSupply.call()) - spec.shares, 0);
    assert.equal(await idea.name.call(), spec.name);
    assert.equal(await idea.symbol.call(), spec.ticker);
  };

  it("should have allocated 100% of the root token supply to the first account", async () => {
    // The very first deployed instance should be the root node. Its
    // information should match that recorded in the respective migration
    // script
    const idea = await Idea.deployed();

    await checkSupply(idea);
  });

  // Token attributes should be as indicated in 2 migration
  it("should have token details equal to those specified in the config file", async () => {
    const idea = await Idea.deployed();

    await checkMetadata(idea, ROOT_IDEA);
  });

  it("should be able to be deployed with some details", async () => {
    // A user should be able to create new ideas as they wish
    const [token, newIdea] = await withTestIdea();

    checkSupply(newIdea, token);
    checkMetadata(newIdea, token);
  });

  // Any user can propose to fund a project
  it("should be able to be funded via a proposal with some details", async () => {
    // Proposals are governed by the parent for which they are forming connections
    const juris = await Idea.deployed();
    const [spec, newIdea] = await withTestIdea();

    // Make a request to fund a new idea with the jurisdiction token that expires in one day
    const prop = await withProp(juris, newIdea);

    // Call to the constructor should have properly set fields
    assert.equal(await prop.governed.call(), juris.address);
    assert.equal(await prop.toFund.call(), newIdea.address);
  });

  // Users can vote on proposals to fund projects
  it("should have a funding proposal that is votable", async () => {
    const juris = await Idea.deployed();
    const [spec, newIdea] = await withTestIdea();
    const prop = await withProp(juris, newIdea);

    // Submit a vote using all of the coins allocated to us to set the funds
    // rate to be 1 token of the jurisdiction token that expires in 2 days,
    // and that can be released once every 24 hours
    let fundsExpiry = new Date((await time.latest()) * 1000);
    fundsExpiry.setDate(fundsExpiry.getDate() + 3);

    // Approve spending for the vote
    await juris.approve(prop.address, ROOT_IDEA.shares);

    // Submit vote
    await prop.vote(
      ROOT_IDEA.shares,
      {
        token: juris.address,
        value: 1,
        intervalLength: 86400,
        expiry: Math.round(fundsExpiry.getTime() / 1000),
        lastClaimed: 0,
        kind: 0
      }
    );
  });

  // Users' votes should be registered after a vote is finalized
  it("should be able to be funded via votes", async () => {
    const juris = await withBlankDAO();
    const [spec, newIdea] = await withTestIdea();
    const prop = await withProp(juris, newIdea);

    let fundsExpiry = new Date((await time.latest()) * 1000);
    fundsExpiry.setDate(fundsExpiry.getDate() + 3);

    await juris.approve(prop.address, ROOT_IDEA.shares);

    await prop.vote(
      ROOT_IDEA.shares,
      {
        token: juris.address,
        value: 1,
        intervalLength: 86400,
        expiry: Math.round(fundsExpiry.getTime() / 1000),
        lastClaimed: 0,
        kind: 0
      }
    );

    // Simulate time passing to demonstrate that users' votes can be finalized:
    // advance time forward by one day
    await time.increase(86400);

    // Any user should be able to finalize a proposal if the vote has ended
    await juris.finalizeProp(prop.address);

    assert.equal((await juris.fundedIdeas.call(newIdea.address)).value, 1);
  });
});
