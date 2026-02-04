import init, * as wasm from "./pkg/mike.js";

console.log("Start.");

await init();

console.log("Halfway there.");

wasm.main(window.innerWidth, window.innerHeight);

console.log("Test?");