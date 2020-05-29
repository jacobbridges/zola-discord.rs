use serenity::model::id::ChannelId;
use serenity::http::client::Http;
use std::fmt::Display;

pub fn devlog(http: impl AsRef<Http>, content: impl Display) {
  let _ = ChannelId(428757417535209472).say(&http, content);
}