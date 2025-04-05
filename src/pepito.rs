use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::env;
use tokio::spawn;

use sse_client::EventSource;

async fn send_discord_message(content: &str) {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Http::new(&token);
    let discord_channel_id = 1263881335479472240;
    let channel_id = ChannelId::new(discord_channel_id);
    if let Err(why) = channel_id.say(http, content).await {
        println!("Error sending message: {:?}", why);
    }
}

async fn on_pepito(_event: sse_client::Event) {
    send_discord_message("recieved pepito").await;
}

async fn on_heartbeat(_event: sse_client::Event) {
    send_discord_message("recieved heartbeat").await;
}

pub async fn start_pepito() {
    spawn(async {
        let event_src = EventSource::new("https://api.thecatdoor.com/sse/v1/events").unwrap();

        event_src.on_open(|| {
            println!("Connection successfull!");
        });

        event_src.add_event_listener("pepito", |event| {
            spawn(async move {
                println!("{:?}", event);
                on_pepito(event).await;
            });
        });

        event_src.add_event_listener("heartbeat", |event| {
            spawn(async move {
                println!("{:?}", event);
                on_heartbeat(event).await;
            });
        });
        println!("Connected to pepito api");
    });
}
