use std::usize;

fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // <length>:<contents>
    if let Some((len,content)) = encoded_value.split_once(":"){
        if let Ok(len) = len.parse::<usize>(){
            return serde_json::Value::String(content[..len].to_string());
        }
    }
    panic!("Unhandled encoded value {}", encoded_value)
}

fn main(){
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];
    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    }
}