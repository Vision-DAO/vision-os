import fs from "fs";
import { create } from "ipfs";

/**
 * An instance of IPFS used for harnessing a test.
 */
export type IPFSClient = ReturnType<Awaited<typeof create>>;

/**
 * The path to the file containing an IPLD schema to parse describing Vision
 * metadata.
 */
export const SCHEMA_PATH: string = process.env.SCHEMA_PATH || "../fixtures/beacon_layer.ipldsch";

/**
 * The path to a directory of .wasm files that should be loaded for testing
 * purposes. No extensive testing of these modules' functionality is performed.
 */
export const MODULES_PATH: string = process.env.MODULES_PATH || "../fixtures/modules/target/wasm32-unknown-unknown/release/";

/**
 * wasm-pack will stubbornly compile the root src/lib.rs for the modules
 * fixture, but we don't want to actually run it.
 */
export const MODULE_IGNORES: string[] = (process.env.MODULE_IGNORES || "beacon_dao_modules").split(" ");

/**
 * The paths to every WASM module that can be used by the testing suite.
 */
export const MODULES: string[] = fs
	.readdirSync(MODULES_PATH)
	.filter((path) =>
		!(path.substring(path.lastIndexOf(".")) in MODULE_IGNORES)
	);

/**
 * The IPLD schema used for validating compatibility with test methodology.
 */
export const SCHEMA: string = fs.readFileSync(SCHEMA_PATH, "utf-8");

/**
 * Executes a test for every WASM module loaded for testing.
 */
export const forAllModules = async (fn: (module: Uint8Array) => Promise<void>) => {
	for (const modPath of MODULES) {
		const modContents = fs.readFileSync(modPath);

		await fn(modContents);
	}
}

/**
 * Returns true if the two uint8arrays are equivalent, or false if they are not.
 */
export const bytesEqual = (a: Uint8Array, b: Uint8Array): boolean => {
	if (a.length != b.length)
		return false;

	for (let i = 0; i < a.length; i++) {
		if (a[i] != b[i])
			return false;
	}

	return true;
};

/**
 * Returns true if the two objects have the same fields and values.
 */
export const objectsEqual = (a: any, b: any): boolean => {
	if (typeof a !== typeof b)
		return false;

	if (Object.keys(a).length < Object.keys(b).length)
		return false;

	for (const field of Object.keys(a)) {
		if (a[field] != b[field])
			return false;
	}

	return true;
}
