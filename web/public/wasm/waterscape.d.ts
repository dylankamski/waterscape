/* tslint:disable */
/* eslint-disable */

/**
 * JavaScript-friendly Agent wrapper
 */
export class WasmAgent {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Export the signing key (for backup)
     */
    exportSigningKey(): Uint8Array;
    /**
     * Create a new agent with the given name
     */
    constructor(name: string);
    /**
     * Get the agent's public identity as JSON
     */
    publicIdentityJson(): string;
    /**
     * Get the agent's fingerprint (first 8 bytes of signing key as hex)
     */
    readonly fingerprint: string;
    /**
     * Get the agent's name
     */
    readonly name: string;
}

/**
 * JavaScript-friendly Waterscape API
 */
export class WasmWaterscape {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Decode a hidden message
     *
     * # Arguments
     * * `receiver` - The receiving agent
     * * `sender_json` - The sender's public identity as JSON
     * * `text` - The text containing the hidden message
     *
     * # Returns
     * The decoded secret message
     */
    static decode(receiver: WasmAgent, sender_json: string, text: string): string;
    /**
     * Encode a secret message for a recipient
     *
     * # Arguments
     * * `sender` - The sending agent
     * * `recipient_json` - The recipient's public identity as JSON
     * * `cover_text` - The visible cover text
     * * `secret` - The secret message to hide
     *
     * # Returns
     * The cover text with hidden encrypted message
     */
    static encode(sender: WasmAgent, recipient_json: string, cover_text: string, secret: string): string;
    /**
     * Check if text contains a hidden message
     */
    static hasHiddenMessage(text: string): boolean;
    /**
     * Extract only the visible text (remove hidden data)
     */
    static visibleText(text: string): string;
}

/**
 * JavaScript-friendly Group wrapper
 */
export class WasmWaterscapeGroup {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Decode a group message
     */
    decode(text: string): string;
    /**
     * Encode a message for the group
     */
    encode(sender: WasmAgent, cover_text: string, secret: string): string;
    /**
     * Create a new group
     *
     * # Arguments
     * * `name` - Group name
     * * `creator` - The agent creating the group
     * * `members_json` - JSON array of member public identities
     */
    constructor(name: string, creator: WasmAgent, members_json: string);
    /**
     * Get the group name
     */
    readonly name: string;
}

/**
 * Initialize panic hook for better error messages in WASM
 */
export function init(): void;

/**
 * Log to browser console
 */
export function log(message: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmagent_free: (a: number, b: number) => void;
    readonly __wbg_wasmwaterscape_free: (a: number, b: number) => void;
    readonly __wbg_wasmwaterscapegroup_free: (a: number, b: number) => void;
    readonly init: () => void;
    readonly log: (a: number, b: number) => void;
    readonly wasmagent_exportSigningKey: (a: number) => [number, number];
    readonly wasmagent_fingerprint: (a: number) => [number, number];
    readonly wasmagent_name: (a: number) => [number, number];
    readonly wasmagent_new: (a: number, b: number) => number;
    readonly wasmagent_publicIdentityJson: (a: number) => [number, number, number, number];
    readonly wasmwaterscape_decode: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly wasmwaterscape_encode: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly wasmwaterscape_hasHiddenMessage: (a: number, b: number) => number;
    readonly wasmwaterscape_visibleText: (a: number, b: number) => [number, number];
    readonly wasmwaterscapegroup_decode: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmwaterscapegroup_encode: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly wasmwaterscapegroup_name: (a: number) => [number, number];
    readonly wasmwaterscapegroup_new: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
