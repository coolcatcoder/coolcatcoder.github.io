/* tslint:disable */
/* eslint-disable */

export function main(width: number, height: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly main: (a: number, b: number) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___closure__destroy___dyn_core_3c29c74c55b07694___ops__function__FnMut__wasm_bindgen_ec76a5ce7af30f31___JsValue____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___closure__destroy___dyn_core_3c29c74c55b07694___ops__function__FnMut__core_3c29c74c55b07694___option__Option_web_sys_9e05a89f398c141a___features__gen_Blob__Blob_____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___convert__closures_____invoke___js_sys_f997586e1e2ee22___Array__web_sys_9e05a89f398c141a___features__gen_ResizeObserver__ResizeObserver_____: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___convert__closures_____invoke___wasm_bindgen_ec76a5ce7af30f31___JsValue_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___convert__closures_____invoke___js_sys_f997586e1e2ee22___Array_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ec76a5ce7af30f31___convert__closures_____invoke______: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
