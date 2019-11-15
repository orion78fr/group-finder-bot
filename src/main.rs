// YEAH I KNOW THIS IS DIRTY (NO ERROR CHECKING, ALL IN ONE FILE), BUT IT'S A QUICK DEMO

use std::env;
use std::fs;
use std::collections::HashMap;

use serenity::{prelude::*,
               model::prelude::*,
               Client,
               http::CacheHttp,
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

group!({
    name: "manage",
    commands: [lang, here, add_game, remove_game, add_category, remove_category],
    options: {
        checks: [Admin],
    },
});

// Use this link to connect your bot https://discordapp.com/oauth2/authorize?client_id=xxxxxx&scope=bot
// Replacing the x with your bot id
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

    let (owner, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => (info.owner.id, info.id),
        Err(why) => panic!("Could not access application info: {:?}", why)
    };
    println!("Bot id : {}", bot_id);
    println!("Owner : {}", owner);

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
fn lang(_ctx: &mut Context, _msg: &Message, mut args: Args) -> CommandResult {
    // TODO allow different langs
    println!("Command lang {:?}", args);
    Ok(())
}

#[command]
fn here(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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
                        ReactionType::Unicode(String::from(TRASHCAN_EMOJI))).unwrap();
            }
            Err(e) => eprintln!("Error ! Cannot answer !")
        };
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.add_category(guild_id, String::from(args.current().unwrap()));
        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);
    }

    Ok(())
}

#[command]
fn add_game(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 3 {
        let answer = MessageBuilder::new()
            .push("Wrong number of arguments.\n")
            .push("Use \"")
            .mention(&bot_id)
            .push(" add_game category_name, game_name, emoji\" to add a game")
            .build();

        match msg.channel_id.say(&ctx.http, &answer) {
            Ok(m) => {
                m.react(&ctx,
                        ReactionType::Unicode(String::from(TRASHCAN_EMOJI))).unwrap();
            }
            Err(e) => eprintln!("Error ! Cannot answer !")
        };
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();
        let category = String::from(args.current().unwrap());
        let game_name = String::from(args.advance().current().unwrap());
        let emoji = args.advance().current().unwrap().parse().unwrap();

        guild_games.add_game(&guild_id, &category, game_name, EmojiId(emoji));

        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);
    }

    Ok(())
}

#[command]
fn remove_game(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 2 {
        let answer = MessageBuilder::new()
            .push("Wrong number of arguments.\n")
            .push("Use \"")
            .mention(&bot_id)
            .push(" remove_game category_name, game_name\" to remove a game")
            .build();

        match msg.channel_id.say(&ctx.http, &answer) {
            Ok(m) => {
                m.react(&ctx,
                        ReactionType::Unicode(String::from(TRASHCAN_EMOJI))).unwrap();
            }
            Err(e) => eprintln!("Error ! Cannot answer !")
        };
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();
        let category = String::from(args.current().unwrap());
        let game_name = args.advance().current().unwrap();

        guild_games.remove_game(&guild_id, &category, game_name);

        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);
    }

    Ok(())
}

#[command]
fn remove_category(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 1 {
        let answer = MessageBuilder::new()
            .push("Wrong number of arguments.\n")
            .push("Use \"")
            .mention(&bot_id)
            .push(" remove_category category_name\" to remove a category")
            .build();

        match msg.channel_id.say(&ctx.http, &answer) {
            Ok(m) => {
                m.react(&ctx,
                        ReactionType::Unicode(String::from(TRASHCAN_EMOJI))).unwrap();
            }
            Err(e) => eprintln!("Error ! Cannot answer !")
        };
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.remove_category(&guild_id, args.current().unwrap());
        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);
    }

    Ok(())
}

fn update_message(ctx: &Context, chan: &ChannelId, msg_to_edit: &MessageId, categories: &HashMap<String, Category>) {
    ctx.http().get_message(chan.0, msg_to_edit.0).unwrap()
        .edit(ctx, |m| m.content(format_post(categories)));
}

static TRASHCAN_EMOJI: &str = "\u{1F5D1}\u{FE0F}";
static YEA_EMOJI: &str = "\u{2714}\u{FE0F}";
static NAY_EMOJI: &str = "\u{274C}";

struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, added_reaction: Reaction) {
        // TODO check rights
        // Only analyze reactions on our own messages, don't care about the rest
        let bot_id = *ctx.data.read().get::<BotId>().unwrap();
        if added_reaction.user_id != bot_id {
            let reacted_msg = added_reaction.message(&ctx.http).unwrap();
            if reacted_msg.author.id == bot_id {
                if added_reaction.emoji == ReactionType::Unicode(String::from(TRASHCAN_EMOJI)) {
                    reacted_msg.delete(&ctx);
                }
            }
        }
    }

    fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {
        // TODO undo presence
    }
}

fn format_post(categories: &HashMap<String, Category>) -> String {
    if categories.is_empty() {
        return String::from("Please add a category.");
    }

    let mut mb = MessageBuilder::new();

    categories.values().for_each(|cat| {
        mb.push_bold_line(cat.name());

        let games = cat.games();
        if games.is_empty() {
            mb.push("Please add a game.");
        } else {
            games.values().for_each(|game| {
                // TODO mb.emoji(game.emoji);
                mb.push(game.emoji.0);

                mb.push(" - ")
                    .push(&game.name)
                    .push("\n");
            });
        }

        mb.push("\n");
    });

    mb.build()
}

struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}

struct GuildGames {
    guild_msg: HashMap<GuildId, (ChannelId, MessageId)>,
    guild_categories: HashMap<GuildId, HashMap<String, Category>>,
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

    pub fn add_game(&mut self, guild_id: &GuildId, category_name: &String, name: String, emoji: EmojiId) {
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

struct Category {
    name: String,
    games: HashMap<String, Game>,
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

    pub fn add_game(&mut self, name: String, emoji: EmojiId) {
        self.games.insert(name.clone(), Game { name, emoji });
    }

    pub fn remove_game(&mut self, name: &str) {
        self.games.remove(name);
    }

    pub fn games(&self) -> &HashMap<String, Game> {
        &self.games
    }
}

struct Game {
    name: String,
    emoji: EmojiId,
}

impl TypeMapKey for GuildGames {
    type Value = GuildGames;
}

// Directly copied from the serenity docs
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