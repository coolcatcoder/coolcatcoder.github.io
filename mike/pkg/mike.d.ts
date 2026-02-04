/* tslint:disable */
/* eslint-disable */

export function main(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly main: () => void;
    readonly wasm_bindgen_6a08c12e0a397c9___closure__destroy___dyn_core_b03932f25916e09f___ops__function__FnMut__core_b03932f25916e09f___option__Option_web_sys_a8b463aacf1ec2f5___features__gen_Blob__Blob_____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_6a08c12e0a397c9___closure__destroy___dyn_core_b03932f25916e09f___ops__function__FnMut__wasm_bindgen_6a08c12e0a397c9___JsValue____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_6a08c12e0a397c9___convert__closures_____invoke___js_sys_58e02ca7e6c55ee6___Array__web_sys_a8b463aacf1ec2f5___features__gen_ResizeObserver__ResizeObserver_____: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_6a08c12e0a397c9___convert__closures_____invoke___js_sys_58e02ca7e6c55ee6___Array_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_6a08c12e0a397c9___convert__closures_____invoke___wasm_bindgen_6a08c12e0a397c9___JsValue_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_6a08c12e0a397c9___convert__closures_____invoke______: (a: number, b: number) => void;
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
