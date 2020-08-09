mod commands;
mod chlog;

use std::{
  collections::HashSet,
  env,
  sync::Arc,
  fs,
};
use serenity::{
  client::bridge::gateway::ShardManager,
  framework::standard::{
    Args,
    HelpOptions,
    CommandGroup,
    CommandResult,
    help_commands,
    StandardFramework,
    macros::{group, help},
  },
  model::{prelude::*, event::ResumedEvent, gateway::Ready},
  prelude::*,
};
use log::{error, info};
use chlog::devlog;
use rand::seq::SliceRandom;
use commands::{
  color::*,
  big::*
};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

impl EventHandler for Handler {
  fn ready(&self, ctx: Context, _: Ready) {
    info!("I am online. Notifying server.");
    devlog(&ctx.http, String::from("I am online."));
  }
  
  fn message(&self, ctx: Context, message: Message) {
    // Ignore messages from bots
    if message.author.bot {
      return;
    }

    // Make a copy of the message and send it to lowercase, for easier matching
    let mut content = message.content.clone();
    content.make_ascii_lowercase();

    // Ignore command messages
    if content.starts_with("!") {
      return;
    }

    // Handle heresy
    if content.contains("heresy") {
      handle_heresy(&ctx, &message, &content);
    }
    
  }

  fn resume(&self, _: Context, _: ResumedEvent) {
    info!("Resumed");
  }
}

fn handle_heresy(ctx: &Context, message: &Message, _content: &String) {

  let heresy_image_directory = if let Ok(heresy_image_directory) = fs::read_dir("assets\\memes\\sense_heresy") { heresy_image_directory } else {
    error!("Could not read heresy image directory assets\\memes\\sense_heresy");
    return;
  };

  let mut heresy_image_paths = Vec::new();
  for f in heresy_image_directory {
    let unwrapped = if let Ok(f) = f { f } else {
      error!("Could not unwrap entity in directory assets\\memes\\sense_heresy");
      continue;
    };
    heresy_image_paths.push(unwrapped.path());
  }

  if heresy_image_paths.len() > 0 {
    let random_path = if let Some(random_path) = heresy_image_paths.choose(&mut rand::thread_rng()) { random_path } else {
      error!("Failed to grab a random image path from vec: {:?}", heresy_image_paths);
      return;
    };

    let _ = message.channel_id.send_files(
      &ctx.http, 
      vec![random_path], 
      |m| { m.content("") }
    );
  }

}

#[group]
#[commands(big)]
struct General;

#[help]
#[individual_command_tip = "Greetings. If you require more information about a specific command, use !help command."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "."]
#[lacking_permissions = "Hide"]
#[wrong_channel = "Strike"]
fn my_help(
  context: &mut Context,
  msg: &Message,
  args: Args,
  help_options: &'static HelpOptions,
  groups: &[&'static CommandGroup],
  owners: HashSet<UserId>
) -> CommandResult {
  help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn main() {
  // This will load the environment variables located at `./.env`, relative to
  // the CWD. See `./.env.example` for an example on how to structure this.
  kankyo::load().expect("Failed to load .env file");

  // Initialize the logger to use environment variables.
  //
  // In this case, a good default is setting the environment variable
  // `RUST_LOG` to debug`.
  env_logger::init();

  let token = env::var("DISCORD_TOKEN")
    .expect("Expected a token in the environment");

  let mut client = Client::new(&token, Handler).expect("Err creating client");

  {
    let mut data = client.data.write();
    data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
  }

  let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
    Ok(info) => {
        let mut owners = HashSet::new();
        owners.insert(info.owner.id);

        (owners, info.id)
    },
    Err(why) => panic!("Could not access application info: {:?}", why),
  };

  client.with_framework(StandardFramework::new()
    .configure(|c| c
      .on_mention(Some(bot_id))
      .owners(owners)
      .prefix("!"))
    .unrecognised_command(|ctx: &mut Context, msg: &Message, unknown_command_name| {
      let _ = msg.channel_id.say(&ctx.http, format!("Nivix has not taught me how to help with '{}'. Blame him.", unknown_command_name));
    })
    .help(&MY_HELP)
    .bucket("taxing", |b| b.delay(5))
    .group(&GENERAL_GROUP)
    .group(&COLOR_GROUP));
    

  if let Err(why) = client.start() {
    error!("Client error: {:?}", why);
  }
}