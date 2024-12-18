use std::{env, error::Error, fs, path};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest_eventsource::{Event, RequestBuilderExt};

use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let openai_api_key_path = match env::var("GPT_OPENAI_API_KEY_PATH") {
        Ok(p) => path::Path::new(&p).to_path_buf(),
        Err(_) => path::Path::new(&env::var("HOME").unwrap()).join(".openai_key.secret"),
    };

    let key = fs::read_to_string(&openai_api_key_path).unwrap();
    let key = key.trim_end();

    let promt = env::args().skip(1).collect::<Vec<String>>().join(" ");
    let content = &serde_json::json!({
            "model": "gpt-4o-2024-11-20",
            "messages": [
                {
                    "role": "system",
                    "content": "Hi, I'm a helpful and very experienced software developer assistant that gives answers straight to the point. What can I help you with?"
                  },
                  {
                    "role": "user",
                    "content": promt.clone()
                  }
            ],
            "temperature": 0.7,
            "stream": true
        }).to_string();

    let client = reqwest::Client::new();

    let secret_key_header_value = format!("Bearer {}", &key);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&secret_key_header_value).unwrap(),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str("application/json").unwrap(),
    );

    let mut req = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .body(content.clone().leak().to_string())
        .try_clone()
        .unwrap()
        .eventsource()
        .unwrap();

    while let Some(event) = req.next().await {
        match event {
            Ok(Event::Message(message)) => {
                handle_chunk(message.data)?;
            }
            Err(err) => {
                req.close();
            }
            _ => (),
        }
    }

    Ok(())
}

fn handle_chunk(msg: String) -> Result<(), Box<dyn Error>> {
    let val = match serde_json::from_str::<serde_json::Value>(&msg) {
        Ok(val) => val,
        Err(err) => {
            return Ok(());
        }
    };

    let content = &val["choices"][0]["delta"]["content"];

    if content.as_str().is_none() {
        return Ok(());
    }

    print!("{}", content.as_str().unwrap());
    Ok(())
}
