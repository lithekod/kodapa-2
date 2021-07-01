use super::Result;

use futures_util::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption};
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::gateway::{Intents, payload::InteractionCreate};

pub async fn handle(token: &str) -> Result<()> {
    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(token, Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

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

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
) -> Result<()> {
    match event {
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        Event::InteractionCreate(box interaction) => {
            handle_interaction(interaction, &http).await?;
        }
        // Other events here...
        event => {
            println!("{:?}", event);
        }
    }

    Ok(())
}

async fn handle_interaction(interaction: InteractionCreate, http: &HttpClient) -> Result<()> {
    match interaction.0 {
        Interaction::Ping(_) => println!("pong (interaction)"),
        Interaction::ApplicationCommand(box ApplicationCommand {
            data: CommandData {
                name: _cmd_name,
                options: cmd_options,
                ..
            },
            id,
            member: _member,
            token,
            ..
        }) => {
            let _title = cmd_options.iter().find_map(|option| {
                if let CommandDataOption::String { name, value } = option {
                    if name == "title" {
                        return Some(value);
                    }
                }
                None
            });
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
            ).await?;
        }
        i => println!("unhandled interaction: {:?}", i),
    }

    Ok(())
}
