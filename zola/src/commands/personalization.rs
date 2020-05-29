#[path = "../chlog.rs"]
mod chlog;

use serenity::prelude::*;
use serenity::model::{
  prelude::*,
};
use serenity::framework::standard::{
  macros::command,
  CommandResult,
  Args,
};
use log::{info, error};
use chlog::devlog;

fn hex_to_rgb(hex: &String) -> Result<u64, u64> {
  let r = if let Ok(r) = u64::from_str_radix(&hex[1..3], 16) { r } else { 
    error!("Failed to parse byte \"r\" from hex {}", hex);
    return Err(0)
  };
  
  let g = if let Ok(g) = u64::from_str_radix(&hex[3..5], 16) { g } else {
    error!("Failed to parse byte \"g\" from hex {}", hex);
    return Err(0)
  };
  
  let b = if let Ok(b) = u64::from_str_radix(&hex[5..7], 16) { b } else {
    error!("Failed to parse byte \"b\" from hex {}", hex);
    return Err(0)
  };
  
  Ok((r << 16) + (g << 8) + b)
}


#[command]
fn color(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {

  match args.single::<String>() {
    Ok(subcommand) => match subcommand.as_ref() {
      "add" => add_color(ctx, &msg, args),
      _ => {let _ = msg.channel_id.say(&ctx.http, "The color command accepts the following subcommands: add, list, and =");}
    },
    Err(_) => {let _ = msg.channel_id.say(&ctx.http, "The color command accepts the following subcommands: add, list, and =");},
  }

  Ok(())
}

fn add_color(ctx: &mut Context, msg: &Message, mut args: Args) {
  // TODO: Check if calling user is Guru or above

  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: color add label hexcode");};
  let send_invalid_hexcode_message = || {let _ = msg.channel_id.say(&ctx.http, "Hexcodes should be a hash followed by 6 hexadecimal characters. e.g. #aba123\nFor more hexcode color examples, see https://htmlcolorcodes.com/color-chart/");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let label = if let Ok(label) = args.single::<String>() { label } else {
    send_usage_message();
    return;
  };

  let hexcode = if let Ok(hexcode) = args.single::<String>() { hexcode } else {
    send_usage_message();
    return;
  };

  if hexcode.len() != 7 || !hexcode.starts_with("#") {
    send_invalid_hexcode_message();
    return;
  };

  if hexcode == "#000000" {
    let _ = msg.channel_id.say(&ctx.http, format!("Discord treats #000000 as \"no color\". I suggest #000001."));
    return;
  }

  let rgb = if let Ok(rgb) = hex_to_rgb(&hexcode) { rgb } else {
    send_invalid_hexcode_message();
    return;
  };

  let guild_id = if let Some(guild_id) = msg.guild_id { guild_id } else {
    send_something_went_wrong_message();
    return;
  };

  match guild_id.create_role(&ctx, |r| r.colour(rgb).name(&label)) {
    Ok(_) => {
      devlog(&ctx.http, format!("INFO: {} created role {} with color <hex:{}, rgb:{}>", msg.author.name, label, hexcode, rgb));
      info!("{} created role {} with color <hex:{}, rgb:{}>", msg.author.name, label, hexcode, rgb);
      let _ = msg.channel_id.say(&ctx.http, format!("Role \"{}\" is now available.", label));
    },
    Err(why) => {
      devlog(&ctx.http, format!("ERROR: Failed to create role {}. {}", label, why));
      error!("Failed to create role {}. {}", label, why);
      let _ = msg.channel_id.say(&ctx.http, format!("Could not create that role at this time. Please note this is a failure on nivix's part, not mine."));
    },
  }
}