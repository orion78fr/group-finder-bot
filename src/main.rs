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
use crate::games_model::{GuildGames, Category};
use crate::utils::format_post;

mod games_model;
mod utils;

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
        data.insert::<GuildGames>(games_model::load().unwrap_or(GuildGames::new()));
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .delimiters(vec![", ", ","]))
        .unrecognised_command(|ctx, msg, unrecognised_command| {
            // For debugging purposes
            println!("Unrecognised message : {}", msg.content);
            let splits: Vec<&str> = msg.content.split_ascii_whitespace().skip(1).collect();
            if splits.len() == 1 {
                let emoji: EmojiIdentifier = splits[0].parse().unwrap();
                println!("Emoji : {} : {}", emoji.name, emoji.id);
                msg.reply(&ctx, format!("<:{}:{}>", emoji.name, emoji.id));
            }
        })
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
            let mut data = ctx.data.write();
            data.get_mut::<GuildGames>().unwrap()
                .set_msg(msg.guild_id.unwrap(), m.channel_id, m.id);
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
        let answer = wrong_argument_message(args.len(), 1,
                                            &bot_id, "add_category",
                                            vec!("category_name"), "add a category");

        message_with_trashcan(ctx, &answer, &msg.channel_id);
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.add_category(guild_id, String::from(args.current().unwrap()));
        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);

        games_model::save(guild_games);
    }

    Ok(())
}

#[command]
fn add_game(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 3 {
        let answer = wrong_argument_message(args.len(), 3,
                                            &bot_id, "add_game",
                                            vec!("category_name", "game_name", "emoji"), "add a game");

        message_with_trashcan(ctx, &answer, &msg.channel_id);
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();
        let category = String::from(args.current().unwrap());
        let game_name = String::from(args.advance().current().unwrap());
        let emoji: EmojiIdentifier = args.advance().current().unwrap().parse().unwrap();

        guild_games.add_game(&guild_id, &category, game_name, emoji);

        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);

        games_model::save(guild_games);
    }

    Ok(())
}

#[command]
fn remove_game(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 2 {
        let answer = wrong_argument_message(args.len(), 2,
                                            &bot_id, "remove_game",
                                            vec!("category_name", "game_name"), "remove a game");

        message_with_trashcan(ctx, &answer, &msg.channel_id);
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();
        let category = String::from(args.current().unwrap());
        let game_name = args.advance().current().unwrap();

        guild_games.remove_game(&guild_id, &category, game_name);

        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);

        games_model::save(guild_games);
    }

    Ok(())
}

#[command]
fn remove_category(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_id = *ctx.data.read().get::<BotId>().unwrap();
    let guild_id = msg.guild_id.unwrap();

    msg.delete(&ctx); // Don't care if failing

    if args.len() != 1 {
        let answer = wrong_argument_message(args.len(), 1,
                                            &bot_id, "remove_category",
                                            vec!("category_name"), "remove a category");

        message_with_trashcan(ctx, &answer, &msg.channel_id);
    } else {
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.remove_category(&guild_id, args.current().unwrap());
        let (chan, msg_to_edit) = guild_games.msg(&guild_id).unwrap();
        let categories = guild_games.categories(&guild_id).unwrap();
        update_message(ctx, chan, msg_to_edit, categories);

        games_model::save(guild_games);
    }

    Ok(())
}

fn message_with_trashcan(ctx: &Context, message_content: &String, channel: &ChannelId) {
    match channel.say(&ctx.http, message_content) {
        Ok(m) => {
            m.react(ctx,
                    ReactionType::Unicode(String::from(TRASHCAN_EMOJI))).unwrap();
        }
        Err(e) => eprintln!("Error ! Cannot answer ! {:?}", e)
    };
}

fn update_message(ctx: &Context, chan: &ChannelId, msg_to_edit: &MessageId, categories: &HashMap<String, Category>) {
    let mut msg = ctx.http().get_message(chan.0, msg_to_edit.0).unwrap();
    msg.edit(ctx, |m| m.content(format_post(categories)));
}

fn wrong_argument_message(args_num: usize,
                          expected_args_num: usize,
                          bot_id: &UserId,
                          command_name: &str,
                          args_name: Vec<&str>,
                          desc: &str) -> String {
    let mut mb = MessageBuilder::new();

    mb.push("Wrong number of arguments (")
        .push(args_num)
        .push(" instead of ")
        .push(expected_args_num)
        .push(")\n");

    mb.push("Use \"")
        .mention(bot_id)
        .push(" ")
        .push(command_name);

    args_name.iter().for_each(|arg| {
        mb.push(" ").push(arg);
    });

    mb.push("\" to ")
        .push(desc)
        .build()
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
            if reacted_msg.author.id == bot_id
                && added_reaction.emoji == ReactionType::Unicode(String::from(TRASHCAN_EMOJI)) {
                reacted_msg.delete(&ctx);
            }
        }
    }

    fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {
        // TODO undo presence
    }

    fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        // Update all messages
        let mut data = ctx.data.write();
        let mut guild_games = data.get_mut::<GuildGames>().unwrap();

        guild_games.msgs().iter()
            .for_each(|(guild_id, (channel_id, message_id))| {
                let categories = guild_games.categories(guild_id).unwrap();
                update_message(&ctx, channel_id, message_id, categories);

                // Refresh reactions
                ctx.http.delete_message_reactions(channel_id.0, message_id.0).unwrap();
                categories.values().flat_map(|cat| cat.games().values())
                    .map(|game| game.emoji())
                    .for_each(|emoji| {
                        ctx.http.create_reaction(channel_id.0, message_id.0,
                                                 &ReactionType::Custom { animated: false, id: emoji.id, name: Some(String::from(&emoji.name)) });
                    })
            });
    }
}

struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
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