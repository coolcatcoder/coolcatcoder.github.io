import init, * as wasm from "./pkg/discord_tools.js";

await init();

function regional_convert() {
    let input = document.getElementById("regional_input").value;
    let no_flags = document.getElementById("no_flags").checked;

    let output = wasm.convert_string_to_regionals(input, no_flags);

    document.getElementById("regional_input").value = output;
}
window.regional_convert = regional_convert;