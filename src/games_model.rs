use std::collections::HashMap;
use serenity::model::id::{GuildId, ChannelId, MessageId, EmojiId};
use serenity::model::misc::EmojiIdentifier;
use std::{fs, io};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GuildGames {
    guild_msg: HashMap<GuildId, (ChannelId, MessageId)>,
    guild_categories: HashMap<GuildId, HashMap<String, Category>>,
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    name: String,
    games: HashMap<String, Game>,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    name: String,
    #[serde(with = "EmojiIdentifierDef")]
    emoji: EmojiIdentifier,
}

impl GuildGames {
    pub fn new() -> GuildGames {
        GuildGames {
            guild_msg: HashMap::new(),
            guild_categories: HashMap::new(),
        }
    }

    pub fn msg(&self, guild_id: &GuildId) -> Option<&(ChannelId, MessageId)> {
        self.guild_msg.get(guild_id)
    }

    pub fn msgs(&self) -> &HashMap<GuildId, (ChannelId, MessageId)> {
        &self.guild_msg
    }

    pub fn categories(&self, guild_id: &GuildId) -> Option<&HashMap<String, Category>> {
        self.guild_categories.get(guild_id)
    }

    pub fn set_msg(&mut self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId) {
        self.guild_msg.insert(guild_id, (channel_id, message_id));
    }

    pub fn add_category(&mut self, guild_id: GuildId, category_name: String) {
        self.guild_categories.entry(guild_id).or_insert(HashMap::new())
            .insert(category_name.clone(), Category::new(category_name));
    }

    pub fn remove_category(&mut self, guild_id: &GuildId, category_name: &str) {
        self.guild_categories.get_mut(guild_id).unwrap()
            .remove(category_name);
    }

    pub fn add_game(&mut self, guild_id: &GuildId, category_name: &String, name: String, emoji: EmojiIdentifier) {
        self.guild_categories.get_mut(guild_id).unwrap()
            .get_mut(category_name).unwrap()
            .add_game(name, emoji);
    }

    pub fn remove_game(&mut self, guild_id: &GuildId, category_name: &str, name: &str) {
        self.guild_categories.get_mut(guild_id).unwrap()
            .get_mut(category_name).unwrap()
            .remove_game(name);
    }
}

impl Category {
    pub fn new(name: String) -> Category {
        Category {
            name,
            games: HashMap::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn add_game(&mut self, name: String, emoji: EmojiIdentifier) {
        self.games.insert(name.clone(), Game { name, emoji });
    }

    pub fn remove_game(&mut self, name: &str) {
        self.games.remove(name);
    }

    pub fn games(&self) -> &HashMap<String, Game> {
        &self.games
    }
}

impl Game {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn emoji(&self) -> &EmojiIdentifier {
        &self.emoji
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "EmojiIdentifier")]
struct EmojiIdentifierDef {
    pub id: EmojiId,
    pub name: String,
}

pub fn save(data: &GuildGames) -> io::Result<()> {
    println!("Saving data...");
    fs::write("data.json", serde_json::to_string(data).unwrap())
}

pub fn load() -> Option<GuildGames> {
    let data = fs::read_to_string("data.json");
    if data.is_err() {
        eprintln!("{}", data.unwrap_err().to_string());
        None
    } else {
        match serde_json::from_str(&data.unwrap()) {
            Ok(r) => Some(r),
            Err(why) => {
                eprintln!("{}", why);
                None
            }
        }
    }
}