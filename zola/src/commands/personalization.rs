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
      "list" => list_colors(ctx, &msg),
      "ls" => list_colors(ctx, &msg),
      "delete" => delete_color(ctx, &msg, args),
      "rm" => delete_color(ctx, &msg, args),
      "set" => set_color(ctx, &msg, args),
      "=" => set_color(ctx, &msg, args),
      _ => {let _ = msg.channel_id.say(&ctx.http, "The color command accepts the following subcommands: add, list, delete and set");}
    },
    Err(_) => {let _ = msg.channel_id.say(&ctx.http, "The color command accepts the following subcommands: add, list, delete and set");},
  }

  Ok(())
}

fn add_color(ctx: &mut Context, msg: &Message, mut args: Args) {
  // TODO: Check if calling user is Guru or above

  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color add label hexcode");};
  let send_invalid_hexcode_message = || {let _ = msg.channel_id.say(&ctx.http, "Hexcodes should be a hash followed by 6 hexadecimal characters. e.g. #aba123\nFor more hexcode color examples, see https://htmlcolorcodes.com/color-chart/");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let label = if let Ok(label) = args.single_quoted::<String>() { label } else {
    send_usage_message();
    return;
  };

  let hexcode = if let Ok(hexcode) = args.single_quoted::<String>() { hexcode } else {
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

  let role_name = String::from("cl:") + &label;
  match guild_id.create_role(&ctx, |r| r.colour(rgb).name(role_name)) {
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

fn list_colors(ctx: &mut Context, msg: &Message) {
  // TODO: Don't just show color names, but show actual colors!
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let guild = if let Some(guild) = msg.guild(&ctx) { guild } else {
    send_something_went_wrong_message();
    return;
  };

  let mut color_list: Vec<String> = Vec::new();

  for (_, role) in guild.read().roles.iter() {
    if role.name.starts_with("cl:") {
      color_list.push(String::from(&role.name[3..]));
    }
  }

  if color_list.len() > 0 {
    let _ = msg.channel_id.say(&ctx.http, format!("These are the available colors: {}", color_list.join(", ")));
  } else {
    let _ = msg.channel_id.say(&ctx.http, "I couldn't find any color roles. Message a Guru for help.");
  }
}

fn delete_color(ctx: &mut Context, msg: &Message, mut args: Args) {
  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color delete label");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let color_label = if let Ok(color_label) = args.single_quoted::<String>() { color_label } else {
    send_usage_message();
    return;
  };

  let locked_guild = if let Some(guild) = msg.guild(&ctx) { guild } else {
    send_something_went_wrong_message();
    return;
  };
  let guild = locked_guild.read();

  let role_name = String::from("cl:") + &color_label;
  let role = if let Some(role) = guild.role_by_name(&role_name) { role } else {
    let _ = msg.channel_id.say(&ctx.http, format!("There is no role with the name {}. You must be mistaken.", &color_label));
    return;
  };

  match guild.delete_role(&ctx.http, role.id) {
    Ok(_) => {
      info!("Color {} has been destroyed by {}", color_label, msg.author.name);
      devlog(&ctx.http, format!("INFO: Color {} has been destroyed by {}", color_label, msg.author.name));
      let _ = msg.channel_id.say(&ctx.http, format!("Color {} has been destroyed.", color_label));
    },
    Err(why) => {
      error!("The color role {:?} could not be deleted for this reason {:?}", role, why);
      devlog(&ctx.http, format!("ERROR: The color role {:?} could not be deleted for this reason {:?}", role, why));
    }
  }

}

fn set_color(ctx: &mut Context, msg: &Message, mut args: Args) {
  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color set label");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let label = if let Ok(label) = args.single_quoted::<String>() { label } else {
    send_usage_message();
    return;
  };

  let guild_id = if let Some(guild_id) = msg.guild_id { guild_id } else {
    send_something_went_wrong_message();
    return;
  };

  let partial_guild = if let Ok(partial_guild) = guild_id.to_partial_guild(&ctx.http) { partial_guild } else {
    send_something_went_wrong_message();
    return;
  };

  let role_name = String::from("cl:") + &label;
  let role = if let Some(role) = partial_guild.role_by_name(&role_name) { role } else {
    let _ = msg.channel_id.say(&ctx.http, format!("No color role exists for label {}.", &label));
    return;
  };

  let mut member = if let Some(member) = msg.member(&ctx) { member } else {
    send_something_went_wrong_message();
    return;
  };

  let member_roles = if let Some(member_roles) = member.roles(&ctx) { member_roles } else {
    send_something_went_wrong_message();
    return;
  };

  for member_role in member_roles {
    if &member_role == role {
      let _ = msg.channel_id.say(&ctx.http, format!("Color {} is already assigned to you.", &label));
      return;
    } else if member_role.name.starts_with("cl:") {
      &member.remove_role(&ctx.http, &member_role);
      let _ = msg.channel_id.say(&ctx.http, format!("Removed current color {}.", &member_role.name));
    }
  }

  match &member.add_role(&ctx.http, &role.id) {
    Ok(_) => {
      let _ = msg.channel_id.say(&ctx.http, format!("Your self-assigned color is now {}", &label));
      info!("{} just set their color to {}", msg.author.name, &label);
    },
    Err(_) => {
      send_something_went_wrong_message();
      error!("Failed to add color {} to you.", label);
      let _ = devlog(&ctx.http, format!("Failed to add color {} to you.", label));
    },
  }

}
