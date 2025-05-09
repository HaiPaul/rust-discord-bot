//! Requires the 'framework' feature flag be enabled in your project's `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
#![allow(deprecated)]
use dotenv::dotenv;
use serenity::framework::standard::{BucketBuilder, Configuration, StandardFramework};
use serenity::http::Http;
use std::env;

mod command_base;
mod emoji_commands;
mod general_commands;
mod picture_commands;
mod moderator_commands;

use command_base::*;
use emoji_commands::*;
use general_commands::*;
use picture_commands::*;
use moderator_commands::*;

#[tokio::main]
async fn main() {
    dotenv().ok(); // load the .env file into the program

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        // Set a function to be called prior to each command execution. This provides the context
        // of the command, the message that was received, and the full name of the command that
        // will be called.
        //
        // Avoid using this to determine whether a specific command should be executed. Instead,
        // prefer using the `#[check]` macro which gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your application if you are
        // fine using nightly Rust. If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_ command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's command could not be
        // found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command can
        // only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        .bucket("pic", BucketBuilder::default().delay(5))
        .await
        // Can't be used more than once per 5 seconds:
        .bucket("emoji", BucketBuilder::default().delay(5))
        .await
        .bucket("mod", BucketBuilder::default().delay(5))
        .await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per
        // channel. Optionally `await_ratelimits` will delay until the command can be executed
        // instead of cancelling the command invocation.
        .bucket(
            "complicated",
            BucketBuilder::default()
                .limit(2)
                .time_span(30)
                .delay(5)
                // The target each bucket will apply to.
                .limit_for(LimitedFor::Channel)
                // The maximum amount of command invocations that can be delayed per target.
                // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                .await_ratelimits(1)
                // A function to call when a rate limit leads to a delay.
                .delay_action(delay_action),
        )
        .await
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&EMOJI_GROUP)
        .group(&PIC_GROUP)
        .group(&MOD_GROUP);

    framework.configure(
        Configuration::new()
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("?")
            // In this case, if "," would be first, a message would never be delimited at ", ",
            // forcing you to trim your arguments if you want to avoid whitespaces at the start of
            // each.
            .delimiters(vec![", ", ","])
            // Sets the bot's owners. These will be used for commands that are owners only.
            .owners(owners),
    );

    // For this example to run properly, the "Presence Intent" and "Server Members Intent" options
    // need to be enabled.
    // These are needed so the `required_permissions` macro works on the commands that need to use
    // it.
    // You will need to enable these 2 options on the bot application, and possibly wait up to 5
    // minutes.
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
