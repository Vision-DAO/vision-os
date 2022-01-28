import detectEthereumProvider from "@metamask/detect-provider";
import Web3 from "web3";
import { provider } from "web3-core";

/*
 * Things this API should be able to do:
 *
 * - Deploy new idea contracts
 * - Create a new proposal for a vote
 * - Get a list of all children of an idea
 * - Get a list of ideas owned by an address
 * - Vote on a proposal
 * - Get the details of an idea from IPFS
 * - Get a list of proposals for an idea
 * - Load a list of detached proposals (no votes yet)
 * - Load a list of detached ideas (no parent yet)
 */

/**
 * Gets a web3 instance from the browser's injected ethereum provider.
 */
export const getWeb3 = async (): Promise<Web3> => new Web3(await detectEthereumProvider() as provider);

/**
 * Gets a list of the user's ethereum accounts.
 */
export const getAccounts = async (w: Web3): Promise<string[]> => await w.eth.requestAccounts();
