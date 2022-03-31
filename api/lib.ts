import detectEthereumProvider from "@metamask/detect-provider";
import Web3 from "web3";
import { provider } from "web3-core";
import type { IPFSHTTPClient } from "ipfs-http-client";
import { AddResult } from "ipfs-core-types/src/root";
import { CID } from "multiformats/cid";
import IdeaContract from "../build/contracts/Idea.json";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth/node_modules/web3-eth-contract";

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

/**
 * Subject details of an idea.
 * Represents the undeployed details of the idea.
 */
export interface IdeaMeta {
	name: string;
	ticker: string;
	shares: number;
	data: string;
};

export type Address = string;

/**
 * Represents a parent-child or child-parent relationship of a finalized or nonfinalized state.
 */
export interface Relationship {
	parent: string;
	child: string;

	/* Whether the relationship is proposed, or active. */
	finalized: boolean;
}

/**
 * Ideas can be funded through minted tokens, or an existing treasury.
 */
export enum FundingKind {
	TREASURY,
	MINT
}

/**
 * Represents a rewards rate, usually corresponding to a relationship.
 */
export interface FundingRate {
	token: string;
	value: number;
	intervalLength: number;
	expiry: Date;
	lastClaimed: Date;
	kind: FundingKind;
}

/**
 * Represents a user's choice for the funding rate, and the weight of their
 * vote, in votes.
 */
export interface Vote {
	rate: FundingRate;
	weight: number;
}

/**
 * Represents a proposed relationship.
 */
export class Proposal {
	private con: Contract;

	public async parent(): Promise<Address> {
		return await this.con.methods.governed.call();
	}

	public async child(): Promise<Address> {
		return await this.con.methods.toFund.call();
	}

	public async votes(): Promise<number> {
		return await this.con.methods.nVoters.call();
	}

	public async expiry(): Promise<Date> {
		// EVM timestamps are Unix timestamps. Convert to a JS date for easy
		// manipulation
		return new Date(await this.con.methods.expiry.call() * 1000);
	}

	public async *votesCast(): AsyncGenerator<Promise<[Address, Vote]>, number, boolean> {
		let nVoters = await this.votes();

		for (let i = 0; i < nVoters; i++) {
			yield this.con.methods.voters.call(i)
				.then((voter: Address) => this.con.methods.refunds.get(voter).then((v: object) => {
					let r = v["rate"];

					let vote: Vote = {
						rate: {
							token: r.token,
							value: r.value,
							intervalLength: r.intervalLength,
							expiry: new Date(r.expiry * 1000),
							lastClaimed: new Date(r.lastClaimed * 1000),
							kind: [FundingKind.TREASURY, FundingKind.MINT][r.kind],
						},
						weight: v["votes"],
					};

					return [voter, vote];
				}));
		}

		return nVoters;
	}
}

/**
 * Represents an Idea deployed to the current blockchain.
 */
export class Idea {
	private meta: IdeaMeta;
	private con: Contract;

	/**
	 * Creates a new lazily-loaded Idea instance, representing an Idea that has been
	 * deployed to a blockchain.
	 */
	constructor(contract: Contract) {
		this.meta = {
			name: contract.methods.ipfsAddr.call(),
			ticker: contract.methods.symbol.call(),
			shares: contract.methods.totalSupply.call(),
			data: contract.methods.ipfsAddr.call()
		};
		this.con = contract;
	}

	/**
	 * TODO: Replace with ceramic integration
	 *
	 * Uploads the specified datum to IPFS via the provided client, returning the
	 * content ID where the datum is stored.
	 */
	public static async uploadIdea(client: IPFSHTTPClient, data: Uint8Array): Promise<CID> {
		return client.add(data).then((r: AddResult) => r.cid);
	}

	/**
	 * Deploys a new instance of the Idea smart contract from the given details,
	 * returning an instance of the wrapper utility class.
	 */
	public static async createIdea(w: Web3, acc: string, idea: IdeaMeta): Promise<Idea> {
		let contract = await (new w.eth.Contract(IdeaContract.abi as AbiItem[]))
			.deploy({ data: "", arguments: [idea.name, idea.ticker, idea.shares, idea.data] })
			.send({ from: acc });
		return new Idea(contract);
	}

	/**
	 * Fetches the static details of the deployed Idea.
	 */
	public details(): IdeaMeta {
		return this.meta;
	}

	/**
	 * Returns a generator yielding the set of Ideas that are funded by this idea.
	 */
	public async *children(): AsyncGenerator<Relationship, number, boolean> {
		let nChildren = await this.con.methods.numChildren.call();

		for (let i = 0; i < nChildren; i++) {
			yield this.con.methods.children.call(i);
		}

		return nChildren;
	}

	public onProposal(proposal: )
}
