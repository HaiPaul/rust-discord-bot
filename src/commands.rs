#![allow(deprecated)] // We recommend migrating to poise, instead of using the standard command framework.
use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
};
use serenity::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, UserId};
use serenity::utils::{content_safe, ContentSafeOptions};
use std::env;
use std::fmt::Write;

pub use serenity::framework::standard::buckets::LimitedFor;
pub use serenity::framework::standard::macros::{command, group, help, hook};
pub use serenity::prelude::*;
pub use std::collections::{HashMap, HashSet};
pub use std::sync::Arc;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(say, commands, pov, zitat, weather)]
pub struct General;

#[group]
#[prefixes("emoji")]
#[commands(cat, dog)]
pub struct Emoji;

#[group]
#[prefixes("pic")]
#[commands(bird)]
pub struct Pic;

#[help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible cases of
// ~~strikethrough-commands~~, but only if `strikethrough_commands_tip_in_{dm, guild}` aren't
// specified. If you pass in a value, it will be displayed instead.
pub async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If the command's name does not
    // exist in the counter, add a default value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
pub async fn after(
    _ctx: &Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => println!("Processed command '{command_name}'"),
        Err(why) => println!("Command '{command_name}' returned error {why:?}"),
    }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{unknown_command_name}'");
}

#[hook]
pub async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    _command_name: &str,
) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (name, amount) in counter {
        writeln!(contents, "- {name}: {amount}")?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role mentions are replaced
// with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command("say")]
#[sub_commands(vallah)]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(x) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                // By default roles, users, and channel mentions are cleaned.
                ContentSafeOptions::default()
                    // We do not want to clean channal mentions as they do not ping users.
                    .clean_channel(false)
                    // If it's a guild channel, we want mentioned users to be displayed as their
                    // display name.
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let content = content_safe(&ctx.cache, x, &settings, &msg.mentions);

            msg.reply(&ctx.http, &content).await?;

            return Ok(());
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
            return Ok(());
        }
    };
}

// sub-command for say
#[command]
async fn vallah(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "Nee").await?;

    Ok(())
}

#[command]
// Adds multiple aliases
#[aliases("kitty", "neko")]
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    Ok(())
}

#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":dog:").await?;

    Ok(())
}

#[command]
#[bucket = "pic"]
async fn bird(ctx: &Context, msg: &Message) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=2);
    let path = format!("images/bird{}.jpg", rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("bird{}.jpg", rng)).await?;
    let _ = match msg
        .channel_id
        .send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().add_file(attachment),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(why) => Err(serenity::all::standard::CommandError::from(why)),
    };
    Ok(())
}

#[command]
async fn pov(ctx: &Context, msg: &Message) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=3);
    let path = format!("images/pov{}.jpg", rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("pov{}.jpg", rng)).await?;
    let _ = match msg
        .channel_id
        .send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().add_file(attachment),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(why) => Err(serenity::all::standard::CommandError::from(why)),
    };
    Ok(())
}

#[command]
async fn zitat(ctx: &Context, msg: &Message) -> CommandResult {
    let zitate_channel_id = ChannelId::new(1290616138308386816);

    let message = &msg.content;

    let f = message.find(' ');

    match f {
        Some(f) => {
            zitate_channel_id
                .say(&ctx.http, message.clone().split_off(f))
                .await?
        }
        None => {
            msg.channel_id
                .say(&ctx.http, "You need to add a zitat!")
                .await?
        }
    };

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Coord {
    lon: f64,
    lat: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    id: u32,
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: u32,
    humidity: u32,
    sea_level: Option<u32>,
    grnd_level: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Wind {
    speed: f64,
    deg: u32,
    gust: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rain {
    #[serde(rename = "1h")]
    one_hour: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Clouds {
    all: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sys {
    r#type: u32,
    id: u32,
    country: String,
    sunrise: u64,
    sunset: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct WeatherData {
    coord: Coord,
    weather: Vec<Weather>,
    base: String,
    main: Main,
    visibility: u32,
    wind: Wind,
    rain: Option<Rain>,
    clouds: Clouds,
    dt: u64,
    sys: Sys,
    timezone: i32,
    id: u64,
    name: String,
    cod: u32,
}

#[command]
#[sub_commands(details)]
async fn weather(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let location = match args.single::<String>() {
        Ok(location) => location,
        Err(_) => {
            msg.reply(ctx, "You need to provide a location!").await?;
            return Err("No location supplied".into());
        }
    };

    let api_key = env::var("WEATHER_API_KEY").expect("Expected a token in the environment");

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        location, api_key
    );

    let response = reqwest::get(&url).await?.text().await?;

    let weather: WeatherData = match serde_json::from_str(&response) {
        Ok(weather) => weather,
        Err(_) => {
            msg.reply(ctx, "Failed to parse weather data!").await?;
            return Err("Parsing error".into());
        }
    };

    let description = match weather.weather[0].description.as_str() {
        "clear sky" => "The sky is clear, and the weather is perfect for any activity",
        "few clouds" => "A few clouds are scattered, but the sky is still clear",
        "scattered clouds" => "Clouds are scattered, but the sky is still clear",
        "broken clouds" => "Clouds are broken, but the sky is still clear",
        "shower rain" => "It's raining, but not too much, and it's perfect for a walk",
        "rain" => "It's raining, and you should stay indoors and play videogames",
        "thunderstorm" => "It's a thunderstorm, which is perfect for a stroll",
        "snow" => "It's snowing, and you should stay indoors and enjoy your snowy day",
        "mist" => "It's a mist, which is perfect for a walk",
        _ => "I'm not sure what the weather is like in this location, but it's always good to check outside",
    };

    msg.reply(ctx, description).await?;
    Ok(())
}

#[command]
async fn details(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let api_key = env::var("WEATHER_API_KEY").expect("Expected a token in the environment");

    let location = match args.single::<String>() {
        Ok(location) => location,
        Err(_) => {
            msg.reply(ctx, "You need to provide a location!").await?;
            return Err("No location supplied".into());
        }
    };

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        location, api_key
    );

    let response = reqwest::get(&url).await?.text().await?;

    let weather: WeatherData = match serde_json::from_str(&response) {
        Ok(weather) => weather,
        Err(_) => {
            msg.reply(ctx, "Failed to parse weather data!").await?;
            return Err("Parsing error".into());
        }
    };

    let temp = format!("{:.1}°C", weather.main.temp);
    let feels_like = format!("{:.1}°C", weather.main.feels_like);
    let humidity = format!("{}%", weather.main.humidity);
    let wind_speed = format!("{:.1} m/s", weather.wind.speed);
    let rain = match weather.rain {
        Some(_) => format!("{:.1} m/s", weather.rain.unwrap().one_hour.unwrap()),
        None => "No rain".to_string(),
    };
    let description = weather.weather[0].description.clone();
    let sunrise = format!(
        "{:02}:{:02}:{:02}",
        (weather.sys.sunrise / 3600) % 24,
        (weather.sys.sunrise / 60) % 60,
        weather.sys.sunrise % 60
    );
    let sunset = format!(
        "{:02}:{:02}:{:02}",
        (weather.sys.sunset / 3600) % 24,
        (weather.sys.sunset / 60) % 60,
        weather.sys.sunset % 60
    );

    msg.reply(
        ctx,
        format!(
            "Temperature: {:}
It feels like: {:}
Humidity: {:}
Wind speed: {:}
Rain?: {:}
Description: {:}
Sunrise: {:}
Sunset: {:}",
            temp, feels_like, humidity, wind_speed, rain, description, sunrise, sunset
        ),
    )
    .await?;

    Ok(())
}
