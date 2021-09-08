use std::convert::{TryFrom, TryInto};

use futures_util::stream::StreamExt;
use tokio::{join, sync::{broadcast, mpsc}};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::{application::callback::{CallbackData, InteractionResponse}, id::ChannelId};
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption};
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::gateway::{Intents, payload::InteractionCreate};

use crate::{agenda::{Agenda, AgendaPoint}, calendar::{self, model::Timestamp}, kodapa};

pub async fn handle(
    token: String,
    _agenda_sender: mpsc::UnboundedSender<AgendaPoint>,
    event_receiver: broadcast::Receiver<kodapa::Event>,
) {
    let http = HttpClient::new(token.clone());

    let _e1 = join!(
        handle_discord_events(&token, &http),
        handle_reminder_events(event_receiver, &http),
    );
}

async fn handle_reminder_events(
    mut receiver: broadcast::Receiver<kodapa::Event>,
    http: &HttpClient,
) {
    while let Ok(event) = receiver.recv().await {
        match event {
            kodapa::Event::Reminder { event } => {
                let channel = ChannelId(697057150106599488);
                http
                    .create_message(channel)
                    .content(&get_meeting_string(&event))
                    .unwrap()
                    .exec()
                    .await
                    .unwrap();
            },
        }
    }
}

async fn handle_discord_events(token: &str, http: &HttpClient) {
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

/// The kinds of interactions we support. Should match the file
/// `register_commands.py` which needs to be run if any changes to commands is
/// to be made.
//TODO: Use hyper instead of Python to register.
enum InteractionCommand {
    Add {
        title: String,
    },
    Agenda,
}

impl TryFrom<CommandData> for InteractionCommand {
    type Error = ();

    fn try_from(data: CommandData) -> Result<Self, Self::Error> {
        match data.name.as_str() {
            "add" => {
                let title = data
                    .options
                    .iter()
                    .find_map(|option| {
                        if let CommandDataOption::String { name, value} = option {
                            if name == "title" {
                                return Some(value);
                            }
                        }
                        None
                    })
                    .ok_or(())?
                    .to_string();
                Ok(InteractionCommand::Add {
                    title,
                })
            }
            "agenda" => Ok(InteractionCommand::Agenda),
            _ => Err(()),
        }
    }
}

async fn handle_interaction(interaction: InteractionCreate, http: &HttpClient) {
    match interaction.0 {
        Interaction::Ping(_) => println!("pong (interaction)"),
        Interaction::ApplicationCommand(application_command) => {
            let ApplicationCommand {
                data,
                id,
                member,
                token,
                ..
            } = *application_command;
            let response = match data.try_into() {
                Ok(InteractionCommand::Add { title }) => {
                    Agenda::push_write(AgendaPoint {
                        title: title.to_string(),
                        adder: member.and_then(|m| m.nick).unwrap_or_else(|| "?".to_string()),
                        timestamp: chrono::Local::now(),
                    });
                    "ok".to_string()
                }
                Ok(InteractionCommand::Agenda) => {
                    get_agenda_string()
                }
                Err(_) => "Error parsing command".to_string(),
            };
            println!("response: {:?}", response);
            http.interaction_callback(
                id,
                &token,
                &InteractionResponse::ChannelMessageWithSource(
                    CallbackData {
                        allowed_mentions: None,
                        components: None,
                        content: Some(response),
                        embeds: Default::default(),
                        flags: None,
                        tts: None,
                    },
                )
            ).exec().await.unwrap();
        }
        i => println!("unhandled interaction: {:?}", i),
    }
}

fn get_meeting_string(event: &calendar::model::events::Event) -> String {
    format!(
        "Meeting at {}!{}\n{}",
        event
            .start()
            .try_into()
            .ok()
            .as_ref()
            .and_then(|dt: &Timestamp| dt.date_time())
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(String::new),
        if let Some(location) = event.location() {
            format!(" Location: {}.", location)
        } else {
            String::new()
        },
        get_agenda_string(),
    )
}

fn get_agenda_string() -> String {
    let points = Agenda::read().points;
    if points.is_empty() {
        "Empty agenda".to_string()
    } else {
        format!(
            "```{}```",
            points
                .iter()
                .map(|point| format!("{}", point))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}
