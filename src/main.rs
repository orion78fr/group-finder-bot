use std::env;
use std::fs;
use std::collections::{HashSet, HashMap};

use serenity::{prelude::*,
               model::prelude::*,
               Client,
               framework::{
                   StandardFramework,
                   standard::{
                       Args,
                       CommandResult,
                       CommandOptions,
                       CheckResult,
                       macros::{check, command, group},
                   },
               },
               utils::MessageBuilder};
use std::hash::{Hash, Hasher};
use serenity::http::CacheHttp;
use serenity::http::routing::RouteInfo::EditMessage;

group!({
    name: "manage",
    commands: [lang, here, add_game, remove_game, add_category, remove_category],
    options: {
        checks: [Admin],
    },
});

// Use this link to connect your bot https://discordapp.com/oauth2/authorize?client_id=xxxxxx&scope=bot
fn main() {
    let token = env::var("DISCORD_TOKEN")
        .or_else(|_| fs::read_to_string(".token"));
    if token.is_err() {
        eprintln!("Token not found. Use `DISCORD_TOKEN` env variable or `.token` file.");
        return;
    }
    let token = token.unwrap();

    let client = Client::new(&token, Handler);
    if let Err(why) = client {
        eprintln!("Cannot create client : {:?}", why);
        return;
    }
    let mut client = client.unwrap();

    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why)
    };
    println!("Bot id : {}", bot_id);
    println!("Owners : {:?}", owners);

    {
        let mut data = client.data.write();

        data.insert::<BotId>(bot_id);
        data.insert::<GuildGames>(GuildGames::new());
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .delimiters(vec![", ", ","]))
        .group(&MANAGE_GROUP));

    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }
}

#[command]
fn lang(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // TODO allow different langs
    println!("Command lang {:?}", args);
    Ok(())
}

#[command]
fn here(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Command here {:?}", args);

    let bot_id = *ctx.data.read().get::<BotId>().unwrap();

    let channel = msg.channel(&ctx.cache).unwrap();

    let answer = MessageBuilder::new()
        .push("Ok ")
        .mention(&msg.author)
        .push(". I will use the ")
        .mention(&channel)
        .push(" channel.\n")

        .push("Use \"")
        .mention(&bot_id)
        .push(" add_category category_name\" to add a category")
        .build();

    match msg.channel_id.say(&ctx.http, &answer) {
        Ok(m) => {
            /*m.react(&ctx,
                    ReactionType::Unicode(String::from(TRASHCAN))).unwrap();*/

            {
                let mut data = ctx.data.write();
                data.get_mut::<GuildGames>().unwrap()
                    .set_msg(msg.guild_id.unwrap(), m.channel_id, m.id);
            }
        }
        Err(e) => eprintln!("Error ! Cannot answer !")
    };

    msg.delete(&ctx); // Don't care if failing

    Ok(())
}

#[command]
fn add_category(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Command add_category {:?}", args);

    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 1 {
        let answer = MessageBuilder::new()
            .push("Wrong number of arguments.\n")
            .push("Use \"")
            .mention(&bot_id)
            .push(" add_category category_name\" to add a category")
            .build();

        match msg.channel_id.say(&ctx.http, &answer) {
            Ok(m) => {
                m.react(&ctx,
                        ReactionType::Unicode(String::from(TRASHCAN))).unwrap();
            }
            Err(e) => eprintln!("Error ! Cannot answer !")
        };
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.add_category(guild_id, String::from(args.current().unwrap()));
        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();

        ctx.http.get_message(chan.0, msg_to_edit.0)
            .unwrap().edit(&ctx, |m| m.content("Edit"));
    }

    Ok(())
}

#[command]
fn add_game(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Command add_game {:?}", args);
    Ok(())
}

#[command]
fn remove_game(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Command remove_game {:?}", args);
    Ok(())
}

#[command]
fn remove_category(context: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Command remove_category {:?}", args);
    Ok(())
}

static TRASHCAN: &str = "\u{1F5D1}\u{FE0F}";

struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, added_reaction: Reaction) {
        // Only analyze reactions on our own messages, don't care about the rest
        let reacted_msg = added_reaction.message(&ctx.http).unwrap();
        if reacted_msg.author.id == *ctx.data.read().get::<BotId>().unwrap() {
            if added_reaction.emoji == ReactionType::Unicode(String::from(TRASHCAN)) {
                reacted_msg.delete(&ctx);
            }
        }
    }

    fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {
        // TODO undo presence
    }
}

struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}

struct GuildGames {
    guild_msg: HashMap<GuildId, (ChannelId, MessageId)>,
    guild_categories: HashMap<GuildId, HashSet<Category>>,
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

    pub fn categories(&self, guild_id: &GuildId) -> Option<&HashSet<Category>> {
        self.guild_categories.get(guild_id)
    }

    pub fn set_msg(&mut self, guild_id: GuildId, channel_id: ChannelId, message_id: MessageId) {
        self.guild_msg.insert(guild_id, (channel_id, message_id));
    }

    pub fn add_category(&mut self, guild_id: GuildId, category_name: String) {
        self.guild_categories.entry(guild_id).or_insert(HashSet::new())
            .insert(Category::new(category_name));
    }
}

struct Category {
    name: String,
    games: HashSet<Game>,
}

impl Category {
    pub fn new(name: String) -> Category {
        Category {
            name,
            games: HashSet::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Category {}

struct Game {
    name: String,
    emoji: EmojiIdentifier,
}

impl Hash for Game {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Game {}

impl TypeMapKey for GuildGames {
    type Value = GuildGames;
}

#[check]
#[name = "Admin"]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}