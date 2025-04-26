use serde_json::Value;
use std::collections::BTreeMap;

fn parse_value(input: &str) -> (Value, &str) {
    let mut chars = input.chars();

    match chars.next() {
        Some('i') => {
            // Integer: i<digits>e
            if let Some(end) = input.find('e') {
                let num = input[1..end].parse::<i64>().unwrap();
                (Value::Number(num.into()), &input[end+1..])
            } else {
                panic!("Invalid bencode integer format");
            }
        }
        Some('l') => {
            // List: l<items>e
            let mut rest = &input[1..]; // skip 'l'
            let mut list = Vec::new();

            while !rest.starts_with('e') {
                let (val, new_rest) = parse_value(rest);
                list.push(val);
                rest = new_rest;
            }

            (Value::Array(list), &rest[1..]) // skip final 'e'
        }
        Some('d') => {
            // Dictionary: d<key><value>e
            let mut rest = &input[1..]; // skip 'd'
            let mut map = BTreeMap::new();

            while !rest.starts_with('e') {
                // Keys must be byte strings
                let (key, new_rest) = parse_value(rest);
                rest = new_rest;

                let key_str = match key {
                    Value::String(s) => s,
                    _ => panic!("Bencode dictionary keys must be strings"),
                };

                let (val, new_rest) = parse_value(rest);
                rest = new_rest;

                map.insert(key_str, val);
            }

            (Value::Object(map.into_iter().collect()), &rest[1..]) // skip final 'e'
        }
        Some(c) if c.is_ascii_digit() => {
            // Byte string: <length>:<content>
            let colon_index = input.find(':').unwrap();
            let len = input[..colon_index].parse::<usize>().unwrap();
            let start = colon_index + 1;
            let end = start + len;
            let string = &input[start..end];
            (Value::String(string.to_string()), &input[end..])
        }
        _ => panic!("Unsupported or invalid bencode format"),
    }
}

fn decode_bencoded_value(encoded: &str) -> Value {
    let (val, _) = parse_value(encoded);
    val
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    }
}
