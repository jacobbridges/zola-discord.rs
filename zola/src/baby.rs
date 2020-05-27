// extern crate http;

// use http::{Request, Response};
// use std::{fs, env, path};

// const DISCORD_API_BASE_URL: &str = "https://discord.com/api";
// const DISCORD_API_VERSION: i8 = 6;


// fn get_discord_api_url() -> String {
//   format!("{}/{}", DISCORD_API_BASE_URL, DISCORD_API_VERSION)
// }


// fn get_discord_api_token() -> String {
//   let mut token_path = env::current_dir()
//     .expect("#1");
//   token_path.push("../token");
  
//   fs::read_to_string(token_path)
//     .expect("#2")
// }


// fn main() {
//   let discord_api_url = get_discord_api_url();
//   let discord_api_token = get_discord_api_token();

//   println!("Discord api url {}", discord_api_url);
//   println!("Bot token: {}", discord_api_token);
//   let response = Request::builder()
//     .method("GET")
//     .uri(discord_api_url)
//     .header("Authorization", format!("Bot {}", discord_api_token))
//     .body(())
//     .unwrap();
    
//   println!("Response from Discord api: {:?}", response)
// }
