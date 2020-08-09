#[path = "../chlog.rs"]
mod chlog;

use serenity::prelude::*;
use serenity::model::{
  prelude::*,
};
use serenity::utils::Colour;
use serenity::framework::standard::{
  macros::{command, group},
  CommandResult,
  CommandError,
  Args,
};
use log::{info, error};
use chlog::devlog;

use std::include_bytes;
use std::collections::HashMap;
use std::path::Path;
use image::{RgbImage, Rgb, ImageFormat};
use rusttype::{Scale, FontCollection};
use imageproc::drawing::draw_text_mut;


const COLOR_ROLE_NAME_MAX_WIDTH: u32 = 20;
const FONT_WIDTH: f32 = 26.8;
const FONT_HEIGHT: f32 = 31.0;


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

#[group]
#[prefixes("color", "cl")]
#[description = "Manage the color of your Discord username."]
#[default_command(info)]
#[commands(add, list, delete, set, info)]
struct Color;


#[command]
#[description = "Information on the color group"]
#[help_available(false)]
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
  let _ = msg.channel_id.say(&ctx.http, "**Color Command Usage:**\n\nList available colors with `!color list`\nSet your color with `!color = label`\nCreate a new color with `!color add label #hexcode`\nDelete a color with `!color delete label`");

  Ok(())
}


#[command]
#[allowed_roles("Emperor", "Guru")]
#[description = "Create a new color."]
#[aliases("create")]
fn add(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  // TODO: Check if calling user is Guru or above

  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color add label hexcode");};
  let send_invalid_hexcode_message = || {let _ = msg.channel_id.say(&ctx.http, "Hexcodes should be a hash followed by 6 hexadecimal characters. e.g. #aba123\nFor more hexcode color examples, see https://htmlcolorcodes.com/color-chart/");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let label = if let Ok(label) = args.single_quoted::<String>() { label } else {
    send_usage_message();
    return Ok(());
  };

  if label.len() as u32 > COLOR_ROLE_NAME_MAX_WIDTH {
    let _ = msg.channel_id.say(&ctx.http, format!("Error. Reduce label to {} letters or less.", COLOR_ROLE_NAME_MAX_WIDTH));
    return Ok(());
  }

  let hexcode = if let Ok(hexcode) = args.single_quoted::<String>() { hexcode } else {
    send_usage_message();
    return Ok(());
  };

  if hexcode.len() != 7 || !hexcode.starts_with("#") {
    send_invalid_hexcode_message();
    return Ok(());
  };

  if hexcode == "#000000" {
    let _ = msg.channel_id.say(&ctx.http, format!("Discord treats #000000 as \"no color\". I suggest #000001."));
    return Ok(());
  }

  let rgb = if let Ok(rgb) = hex_to_rgb(&hexcode) { rgb } else {
    send_invalid_hexcode_message();
    return Ok(());
  };

  let guild = if let Some(guild) = msg.guild(&ctx) { guild } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab guild from msg.guild().".to_string()));
  };
  let guild = guild.read();

  let colors = get_colors(&guild.roles);
  if colors.contains_key(&label) {
    let _ = msg.channel_id.say(&ctx.http, format!("That color label is already taken! Try another one."));
    return Ok(());
  }
  
  let color_role_position = if let Ok(position) = get_starting_color_role_position(&guild.roles) { position } else {
    send_something_went_wrong_message();
    devlog(&ctx.http, format!("ERROR: Could not find starting position for color roles. Maybe missing color role marker?"));
    return Err(CommandError("Could not find starting position for color roles. Maybe missing color role marker?".to_string()));
  };

  let role_name = String::from("cl:") + &label;
  match &guild.create_role(&ctx, |r| r.mentionable(false).colour(rgb).name(role_name).position(color_role_position as u8)) {
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

  Ok(())
}

#[command]
#[bucket = "taxing"]
#[description = "Preview all available colors."]
#[aliases("ls", "preview")]
fn list(ctx: &mut Context, msg: &Message) -> CommandResult {
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let guild = if let Some(guild) = msg.guild(&ctx) { guild } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab guild from msg.guild().".to_string()))
  };

  let colors = get_colors(&guild.read().roles);
  if colors.len() == 0 {
    let _ = msg.channel_id.say(&ctx.http, "I couldn't find any color roles. Message a Guru for help.");
    return Ok(())
  }

  match generate_preview_of_color_roles(colors) {
    Ok(path) => {
      let paths = vec![path];
      let _ = msg.channel_id.send_files(&ctx.http, paths, |m| {
        m.content("")
      });
    },
    Err(why) => {
      error!("Failed to generate image: {}", why);
      devlog(&ctx.http, format!("ERROR: Failed to generate image: {}", why));
    }
  }

  Ok(())
}

#[command]
#[allowed_roles("Emperor", "Guru")]
#[description = "Delete a color."]
#[aliases("rm", "remove")]
fn delete(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color delete label");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let color_label = if let Ok(color_label) = args.single_quoted::<String>() { color_label } else {
    send_usage_message();
    return Ok(());
  };

  let locked_guild = if let Some(guild) = msg.guild(&ctx) { guild } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab guild from msg.guild().".to_string()))
  };
  let guild = locked_guild.read();

  let role_name = String::from("cl:") + &color_label;
  let role = if let Some(role) = guild.role_by_name(&role_name) { role } else {
    let _ = msg.channel_id.say(&ctx.http, format!("There is no role with the name {}. You must be mistaken.", &color_label));
    return Ok(());
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
  
  Ok(())
}

#[command]
#[description = "Pick a color."]
#[aliases("=")]
fn set(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  let send_usage_message = || {let _ = msg.channel_id.say(&ctx.http, "Usage: !color set label");};
  let send_something_went_wrong_message = || {let _ = msg.channel_id.say(&ctx.http, "I could not perform the task. Making a note for future improvement.");};

  let label = if let Ok(label) = args.single_quoted::<String>() { label } else {
    send_usage_message();
    return Ok(());
  };

  let guild_id = if let Some(guild_id) = msg.guild_id { guild_id } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab guild_id from msg.guild_id.".to_string()))
  };

  let partial_guild = if let Ok(partial_guild) = guild_id.to_partial_guild(&ctx.http) { partial_guild } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to convert guild_id to PartialGuild.".to_string()))
  };

  let role_name = String::from("cl:") + &label;
  let role = if let Some(role) = partial_guild.role_by_name(&role_name) { role } else {
    let _ = msg.channel_id.say(&ctx.http, format!("No color role exists for label {}.", &label));
    return Ok(());
  };

  let mut member = if let Some(member) = msg.member(&ctx) { member } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab member from msg.member().".to_string()))
  };

  let member_roles = if let Some(member_roles) = member.roles(&ctx) { member_roles } else {
    send_something_went_wrong_message();
    return Err(CommandError("Failed to grab roles from member.roles().".to_string()))
  };

  for member_role in member_roles {
    if &member_role == role {
      let _ = msg.channel_id.say(&ctx.http, format!("Color {} is already assigned to you.\nIf the color change hasn't taken effect, try typing in a different channel.", &label));
      return Ok(());
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

  Ok(())
}

fn get_starting_color_role_position(roles: &HashMap<RoleId, Role>) -> Result<i64, u8> {
  for (_, role) in roles {
    if role.name == "--colors-start-here--" {
      return Ok(role.position);
    }
  }

  Err(0)
}

fn get_colors(roles: &std::collections::HashMap<serenity::model::id::RoleId, serenity::model::guild::Role>) -> HashMap<String, Colour> {
  let mut color_map: HashMap<String, Colour> = HashMap::new();

  for (_, role) in roles.iter() {
    if role.name.starts_with("cl:") {
      color_map.insert(
        String::from(&role.name[3..]),
        role.colour
      );
    }
  }

  color_map
}

fn generate_preview_of_color_roles(colors: HashMap<String, Colour>) -> Result<&'static Path, u8> {
  if colors.len() == 0 {
    return Err(0)
  }

  let image_width = FONT_WIDTH * COLOR_ROLE_NAME_MAX_WIDTH as f32;
  let image_height = FONT_HEIGHT * colors.len() as f32;

  info!("Creating image with dimensions {}, {}", image_width, image_height);

  let mut image = RgbImage::from_pixel(image_width.ceil() as u32, image_height.ceil() as u32, Rgb([54, 57, 63]));

  let font_bytes = Vec::from(include_bytes!("..\\..\\assets\\Roboto-Bold.ttf") as &[u8]);
  let font_collection = if let Ok(font_collection) = FontCollection::from_bytes(font_bytes) { font_collection } else {
    error!("Failed to parse FontCollection at ..\\..\\assets\\Roboto-Bold.ttf");
    return Err(0);
  };

  let font = if let Ok(font) = font_collection.into_font() { font } else {
    error!("Failed to convert FontCollection (..\\..\\assets\\Roboto-Bold.ttf) into a single font");
    return Err(0);
  };

  let scale = Scale {
    x: FONT_WIDTH,
    y: FONT_HEIGHT,
  };
  
  let mut index = 0;
  for (label, color) in colors.iter() {
    draw_text_mut(
      &mut image,
      (|c: &Colour| { Rgb([c.r(), c.g(), c.b()]) })(&color),
      0,
      (index as f32 * FONT_HEIGHT).ceil() as u32,
      scale,
      &font,
      &label,
    );
    index = index + 1;
  }

  let output_path = Path::new("uploads\\color-preview.png");
  match image.save_with_format(&output_path, ImageFormat::Png) {
    Ok(_) => (),
    Err(why) => {
      error!("{}", why);
      return Err(0)
    }
  }

  Ok(output_path)
}
