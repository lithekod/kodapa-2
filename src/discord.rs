use futures_util::stream::StreamExt;
use tokio::sync::{broadcast, mpsc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption};
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::gateway::{Intents, payload::InteractionCreate};

use crate::{agenda::{Agenda, AgendaPoint}, kodapa};

pub async fn handle(
    token: &str,
    _agenda_sender: mpsc::UnboundedSender<AgendaPoint>,
    _event_receiver: broadcast::Receiver<kodapa::Event>,
) {
    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(token, Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await
        .unwrap();

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = HttpClient::new(token);

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, http.clone()));
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
) {
    match event {
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::InteractionCreate(interaction) => {
            handle_interaction(*interaction, &http).await;
        }
        // Other events here...
        event => {
            println!("{:?}", event);
        }
    }
}

async fn handle_interaction(interaction: InteractionCreate, http: &HttpClient) {
    match interaction.0 {
        Interaction::Ping(_) => println!("pong (interaction)"),
        Interaction::ApplicationCommand(application_command) => {
            let ApplicationCommand {
                data: CommandData {
                    name: _cmd_name,
                    options: cmd_options,
                    ..
                },
                id,
                member: _member,
                token,
                ..
            } = *application_command;
            let title = cmd_options.iter().find_map(|option| {
                if let CommandDataOption::String { name, value } = option {
                    if name == "title" {
                        return Some(value);
                    }
                }
                None
            });
            if let Some(title) = title {
                Agenda::push_write(AgendaPoint {
                    title: title.to_string(),
                    adder: "?".to_string(),
                });
            }
            http.interaction_callback(
                id,
                token,
                InteractionResponse::ChannelMessageWithSource(
                    CallbackData {
                        allowed_mentions: None,
                        flags: None,
                        tts: None,
                        content: Some(format!("ok")),
                        embeds: Default::default(),
                    },
                )
            ).await.unwrap();
        }
        i => println!("unhandled interaction: {:?}", i),
    }
}
