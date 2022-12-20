import { CID } from "multiformats/cid";

/**
 * A basic payload for the Beacon DAO that specifies a void entrypoint, start.
 */
export interface Payload {
	start(): void;

	// Some payloads have a poll() function (scheduler)
	poll(): void | undefined;
}

/**
 * A JavaScript loader that consumes a WASM module and produces a JS wrapper
 * that can call the entrypoint method.
 */
export interface PayloadLoader {
	default: (module: Uint8Array) => Promise<Payload>;
}

/**
 * Binary representation of a WASM program.
 */
export interface IdeaPayload {
	loader: CID;
	module: CID;
}

/**
 * Content, and program payload of an Idea on Vision.
 */
export interface IdeaMetadata {
	title: string;
	description: string;
	payload: CID[];
}

/**
 * Address of the intended new payload specified by a proposal.
 */
export type ProposalPayload = CID;

/**
 * Content of a proposal itself.
 */
export interface ProposalMetadata {
	title: string;
	description: string;
}
