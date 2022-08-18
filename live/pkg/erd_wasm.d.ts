/* tslint:disable */
/* eslint-disable */
/**
* @param {string} erd_script
* @returns {any}
*/
export function compile_erd(erd_script: string): any;
/**
* @param {string} erd_script
* @param {string} sql_dbms
* @returns {any}
*/
export function compile_physical(erd_script: string, sql_dbms: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly compile_erd: (a: number, b: number) => number;
  readonly compile_physical: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
