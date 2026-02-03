use std::fmt::Write;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert_string_to_regionals(input: &str, no_flags: bool) -> String {
    let mut output = String::new();
    for character in input.chars() {
        if character.is_alphabetic() {
            write!(
                &mut output,
                ":regional_indicator_{}:",
                character.to_ascii_lowercase()
            )
            .unwrap();
            // A zero-width space prevents flags from forming, while not changing how it renders.
            if no_flags {
                output.push('\u{200B}');
            }
        } else if character.is_ascii_digit() {
            write!(&mut output, ":number_{character}:").unwrap();
        } else {
            output.push(character);
        }
    }
    output
}

// pub fn blah(input: &str) {
//     enum Status {
//         Original(char),
//         Insert(&'static str),
//         Null,
//     }
//     let list = vec![("ng", "ng"), ("a", "regional_indicator_n")];

//     let mut input: Vec<Status> = input.chars().map(Status::Original).collect();

//     for (to_find, replace_with) in list {
//         let Some((index_start, index_end)) = input.windows(to_find.len()).filter_map(|status| {
//             let mut equal = true;
//             for (status, to_find) in status.iter().zip(to_find.chars()) {
//                 match status {
//                     Status::Original(character) => {
//                         if *character != to_find {
//                             equal = false;
//                             break;
//                         }
//                     }
//                     _ => {
//                         equal = false;
//                         break;
//                     }
//                 }
//             }

//             None
//         }).next() else {
//             continue;
//         };
//     }
// }
