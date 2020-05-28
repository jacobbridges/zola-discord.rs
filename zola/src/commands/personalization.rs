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


#[command]
fn color(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
  // if args.is_empty() {
  //   match msg.channel_id.say(&ctx.http, "The color command accepts the following subcommands: add, list, and =") {
  //     Err(why) => error!("{}", why),
  //     Ok(_) => (),
  //   }
  //   return Ok(())
  // }

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

  match args.single::<String>() {
    Ok(label) => match args.single::<String>() {
      Ok(hexcode) => {
        if hexcode.len() != 7 || !hexcode.starts_with("#") {
          let _ = msg.channel_id.say(&ctx.http, format!("{} is not a valid hexcode!", hexcode));
          return;
        }
        match &hexcode[1..].parse::<u64>() {
          Ok(code) => match msg.guild_id {
            Some(guild_id) => match guild_id.create_role(ctx, |r| r.colour(*code).name(&label)) {
              Ok(r) => {info!("Created role {}. colour: {}", r, code);},
              Err(why) => {error!("Failed to create role {}. {}", label, why);},
            }
            None => {error!("Failed to fetch guild_id for msg {:?}", msg);},
          },
          Err(_) => {let _ = msg.channel_id.say(&ctx.http, format!("{} is not a valid hexcode!", hexcode));},
        }
      }
      Err(_) => {let _ = msg.channel_id.say(&ctx.http, "Usage: color add label #000000");},
    },
    Err(_) => {let _ = msg.channel_id.say(&ctx.http, "Usage: color add label #000000");},
  }
}
