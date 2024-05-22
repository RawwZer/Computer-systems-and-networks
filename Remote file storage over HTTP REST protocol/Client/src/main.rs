use std::io;
use reqwest::Client;
use tokio;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    loop {
        println!("Введите HTTP метод (e.g., GET, POST):");
        let mut method = String::new();
        io::stdin().read_line(&mut method)?;

        println!("Введите URL:");
        let mut url = String::new();
        io::stdin().read_line(&mut url)?;

        println!("Введите тело запроса:");
        let mut body = String::new();
        io::stdin().read_line(&mut body)?;

        // Отправка запроса на сервер
        let response = client
            .request(reqwest::Method::from_bytes(method.trim().as_bytes())?, url.trim())
            .header(reqwest::header::CONTENT_TYPE, "application/json") // Установка заголовка Content-Type
            .body(body.trim().to_owned())
            .send()
            .await?;    

        let status = response.status();
        let text = response.text().await?;

        println!("\nResponse status code: {}\n", status);
        println!("Response body:\n{}\n", text);
    }
}
