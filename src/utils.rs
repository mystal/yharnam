use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use intl_pluralrules::{PluralCategory, PluralRules, PluralRuleType};
use unic_langid::LanguageIdentifier;

const FORMAT_FUNCTION_VALUE_PLACEHOLDER: &str = "<VALUE PLACEHOLDER>";

/// Expands all [format functions](https://yarnspinner.dev/docs/syntax#format-functions)
/// in a given string, using pluralisation rules specified by the given locale (as an IETF
/// BCP-47 language tag).
///
/// # Panics
/// When the string contains a `plural` or `ordinal` format function, but the
/// specified value cannot be parsed as a number.
pub fn expand_format_functions(input: &str, locale_code: &str) -> String {
    let (mut line_with_replacements, format_functions) = parse_format_functions(input);

    let langid: LanguageIdentifier = locale_code.parse().unwrap();

    let ordinal_rules = PluralRules::create(langid.clone(), PluralRuleType::ORDINAL).unwrap();
    let cardinal_rules = PluralRules::create(langid.clone(), PluralRuleType::CARDINAL).unwrap();

    for (i, function) in format_functions.iter().enumerate() {
        // Get the key str to look up in the function data.
        let data_key = match function.kind {
            FormatFunctionKind::Select => &function.value,
            FormatFunctionKind::Plural => {
                let value: f64 = function.value.parse()
                    .unwrap_or_else(|_| panic!("Error while pluralising line '{}': '{}' is not a number", input, &function.value));
                let plural_case = cardinal_rules.select(value).unwrap();
                get_plural_case_str(plural_case)
            }
            FormatFunctionKind::Ordinal => {
                let value: f64 = function.value.parse()
                    .unwrap_or_else(|_| panic!("Error while pluralising line '{}': '{}' is not a number", input, &function.value));
                let plural_case = ordinal_rules.select(value).unwrap();
                get_plural_case_str(plural_case)
            }
        };

        let mut replacement = function.data.get(data_key)
            .cloned()
            .unwrap_or_else(|| format!("<no replacement for {}>", data_key));

        // Insert the value if needed
        replacement = replacement.replace(FORMAT_FUNCTION_VALUE_PLACEHOLDER, &function.value);

        line_with_replacements = line_with_replacements.replacen(&format!("{{{}}}", i), &replacement, 1);
    }

    line_with_replacements
}

fn get_plural_case_str(plural_case: PluralCategory) -> &'static str {
    match plural_case {
        PluralCategory::ZERO => "zero",
        PluralCategory::ONE => "one",
        PluralCategory::TWO => "two",
        PluralCategory::FEW => "few",
        PluralCategory::MANY => "many",
        PluralCategory::OTHER => "other",
    }
}

#[derive(Debug, PartialEq, Eq)]
enum FormatFunctionKind {
    Select,
    Plural,
    Ordinal,
}

impl Default for FormatFunctionKind {
    fn default() -> Self {
        Self::Select
    }
}

#[derive(Debug, Default)]
struct ParsedFormatFunction {
    kind: FormatFunctionKind,
    value: String,
    data: HashMap<String, String>,
}

fn parse_format_functions(input: &str) -> (String, Vec<ParsedFormatFunction>) {
    // TODO: Do we wanna iterate over grapheme clusters instead??
    let mut chars = input.chars().peekable();

    let mut line_with_replacements = String::with_capacity(input.len());

    let mut parsed_functions = Vec::new();

    // Read the entirety of the line
    while let Some(c) = chars.next() {
        if c != '[' {
            // plain text!
            line_with_replacements.push(c);
            continue;
        }

        // the start of a format function!
        let mut function = ParsedFormatFunction::default();

        // Structure of a format function:
        // [ name "value" key1="value1" key2="value2" ]

        // Ensure that only valid function names are used
        function.kind = match expect_id(&mut chars).as_ref() {
            "select" => FormatFunctionKind::Select,
            "plural" => FormatFunctionKind::Plural,
            "ordinal" => FormatFunctionKind::Ordinal,
            name => panic!("Invalid formatting function {} in line \"{}\"", name, input),
        };

        function.value = expect_string(&mut chars);

        // parse and read the data for this format function
        loop {
            consume_whitespace(&mut chars, false);

            if let Some(']') = chars.peek() {
                // we're done adding parameters
                break;
            }

            // this is a key-value pair
            let key = expect_id(&mut chars);
            expect_character(&mut chars, '=');
            let value = expect_string(&mut chars);

            if function.data.contains_key(&key) {
                panic!("Duplicate value '{}' in format function inside line \"{}\"", &key, input)
            }

            function.data.insert(key, value);
        }

        // We now expect the end of this format function
        expect_character(&mut chars, ']');

        // reached the end of this function; add it to the
        // list
        parsed_functions.push(function);

        // and add a placeholder for this function's value
        line_with_replacements.push_str(&format!("{{{}}}", parsed_functions.len() - 1));
    }

    (line_with_replacements, parsed_functions)
}

// id = [_\w][\w0-9_]*
fn expect_id(chars: &mut Peekable<Chars>) -> String {
    consume_whitespace(chars, false);

    let mut id_string = String::new();

    // Read the first character, which must be a letter
    let mut next_char = chars.next()
        .unwrap();

    if next_char.is_alphabetic() || next_char == '_' {
        id_string.push(next_char);
    } else {
        panic!("Expected an identifier inside a format function in line");
    }

    // Read zero or more letters, numbers, or underscores
    loop {
        let temp_next = chars.peek();
        if temp_next.is_none() {
            break;
        }
        next_char = *temp_next.unwrap();

        if next_char.is_alphanumeric() || next_char == '_' {
            id_string.push(next_char);
            chars.next(); // consume it
        } else {
            // no more
            break;
        }
    }
    return id_string;
}

// string = " (\"|\\|^["])* "
fn expect_string(chars: &mut Peekable<Chars>) -> String {
    consume_whitespace(chars, false);

    let mut string = String::new();

    let mut next_char = chars.next().unwrap();
    if next_char != '"' {
        panic!("Expected a string inside a format function in line");
    }

    loop {
        next_char = chars.next().unwrap();

        if next_char == '"' {
            // end of string - consume it but don't
            // append to the final collection
            break;
        } else if next_char == '\\' {
            // an escaped quote or backslash
            let escaped_char = chars.next().unwrap();
            if escaped_char == '\\' || escaped_char == '"' || escaped_char == '%' {
                string.push(escaped_char);
            }
        } else if next_char == '%' {
            string.push_str(FORMAT_FUNCTION_VALUE_PLACEHOLDER);
        } else {
            string.push(next_char);
        }

    }

    return string;
}

// Consume a character, and throw an exception if it
// isn't the one we expect.
fn expect_character(chars: &mut Peekable<Chars>, expected_char: char) {
    consume_whitespace(chars, false);

    let next_char = chars.next();
    if next_char != Some(expected_char) {
        panic!("Expected a {} inside a format function in line", expected_char);
    }
}

// Read and discard all whitespace until we hit
// something that isn't whitespace.
fn consume_whitespace(chars: &mut Peekable<Chars>, allow_end_of_line: bool) {
    loop {
        let next_char = chars.peek();
        if next_char.is_none() && !allow_end_of_line {
            panic!("Unexpected end of line inside a format function in line");
        }
        let next_char = *next_char.unwrap();

        if next_char.is_whitespace() {
            // consume it and continue
            chars.next();
        } else {
            // no more whitespace ahead; don't
            // consume it, but instead stop eating
            // whitespace
            return;
        }
    }
}
