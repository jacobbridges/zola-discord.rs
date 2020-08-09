#[path = "../chlog.rs"]
mod chlog;

use serenity::prelude::*;
use serenity::model::{
  prelude::*,
};
use serenity::framework::standard::{
  macros::{command, group},
  CommandResult,
  CommandError,
  Args,
};
use log::{info, error};
use chlog::devlog;
use std::path::Path;
use reqwest;
use std::io;
use std::fs::File;

use serenity::utils::{parse_emoji, parse_username};


#[command]
#[description = "Magnify emojis or user avatars"]
pub fn big(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !big <emoji or user>");};
  let send_native_emoji_message = || {let _ = msg.channel_id.say(&ctx.http, "Cannot magnify native emojis, like :tada:.");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let target = if let Ok(target) = args.single::<String>() { target } else {
    send_usage_message();
    return Ok(())
  };

  let target_chars: Vec<char> = target.chars().collect();
  if target_chars.len() == 1 && !target_chars[0].is_ascii() {
    info!("{}", target);
    devlog(&ctx.http, format!("{}", target));
    send_native_emoji_message();
    return Ok(())
  }

  match parse_emoji(&target) {
    Some(emoji) => {
      let _ = msg.channel_id.say(&ctx.http, emoji.url());
      return Ok(())
    },
    None => {}
  };

  let user_id = if let Some(user_id) = parse_username(&target) { user_id } else {
    send_usage_message();
    return Ok(())
  };

  let user_id = UserId::from(user_id);

  let user = if let Ok(user) = user_id.to_user(&ctx) { user } else {
    send_something_went_wrong_message();
    return Ok(())
  };

  match user.avatar_url() {
    Some(url) => {
      let _ = msg.channel_id.say(&ctx.http, url);
      return Ok(())
    },
    None => {}
  }

  Ok(())
}

// fn enlarge_image(url: &String) -> Result<Path, u8> {
//   let mut resp = if let Ok(resp) = reqwest::get(url) { resp } else {
//     error!("Failed to download file at {}", url);
//     return Err(0)
//   };

//   let mut f = if let Ok(f) = File::create("downloads/temp") { f } else {
//     error!("Failed to create file at downloads/temp");
//     return Err(0)
//   };

//   let copy_result = if let Ok(copy_result) = io::copy(&mut resp, &mut f) { copy_result } else {
//     error!("Failed to copy image data into file at downloads/temp");
//     return Err(0)
//   };

  
// }