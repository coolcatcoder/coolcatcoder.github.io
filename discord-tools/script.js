function regional_convert() {
    let input = document.getElementById("regional_input").value;
    let no_flags = document.getElementById("no_flags").checked;
    console.log(no_flags);

    let output = ""
    for (let i = 0; i < input.length; i++) {
        let input_char = input.charAt(i);
        if (/[a-zA-Z]/.test(input_char)) {
            output += ":regional_indicator_" + input_char.toLowerCase() + ":";
            if (no_flags) {
                output += '​';
            }
            console.log(input_char);
        } else if (/\d/.test(input_char)) {
            output += output += ":number_" + input_char.toLowerCase() + ":";;
        } else if (input_char == ':') {
            output += "\:";
        } else {
            output += input_char;
        }
        
    }
    document.getElementById("regional_input").value = output;
}