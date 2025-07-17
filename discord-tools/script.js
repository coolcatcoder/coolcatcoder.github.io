function regional_convert() {
    let input = document.getElementById("regional_input").value;
    let output = ""
    for (let i = 0; i < input.length; i++) {
        output += ":regional_indicator_" + input.charAt(i) + ":";
    }
    document.getElementById("regional_input").value = output;
}