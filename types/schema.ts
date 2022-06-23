import { CID } from "multiformats/cid";

/**
 * Binary representation of a WASM program.
 */
export type IdeaPayload = Uint8Array;

/**
 * Content, and program payload of an Idea on Vision.
 */
export interface IdeaMetadata {
	title: string,
	description: string,
	payload: CID,
}

/**
 * Address of the intended new payload specified by a proposal.
 */
export type ProposalPayload = CID;

/**
 * Content of a proposal itself.
 */
export interface ProposalMetadata {
	title: string,
	description: string,
}
