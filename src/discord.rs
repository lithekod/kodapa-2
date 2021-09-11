use std::convert::{TryFrom, TryInto};

use futures_util::stream::StreamExt;
use tokio::{join, sync::{broadcast, mpsc}};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::{Client as HttpClient, request::AuditLogReason};
use twilight_model::{application::callback::{CallbackData, InteractionResponse}, id::{ChannelId, RoleId}};
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
    let secret_channel = ChannelId(
        std::env::var("DISCORD_SECRET_CHANNEL")
            .expect("missing DISCORD_SECRET_CHANNEL")
            .parse()
            .unwrap()
    );
    let meetup_role = RoleId(
        std::env::var("DISCORD_MEETUP_ROLE_ID")
            .expect("missing DISCORD_MEETUP_ROLE_ID")
            .parse()
            .unwrap()
    );

    let _e1 = join!(
        handle_discord_events(&token, &http, secret_channel, meetup_role),
        handle_reminder_events(event_receiver, &http, secret_channel),
    );
}

async fn handle_reminder_events(
    mut receiver: broadcast::Receiver<kodapa::Event>,
    http: &HttpClient,
    secret_channel: ChannelId,
) {
    while let Ok(event) = receiver.recv().await {
        match event {
            kodapa::Event::Reminder { event } => {
                http
                    .create_message(secret_channel)
                    .content(&get_meeting_string(&event))
                    .unwrap()
                    .exec()
                    .await
                    .unwrap();
            },
        }
    }
}

async fn handle_discord_events(token: &str, http: &HttpClient, secret_channel: ChannelId, meetup_role: RoleId) {
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

        tokio::spawn(handle_event(shard_id, event, http.clone(), secret_channel, meetup_role));
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
    secret_channel: ChannelId,
    meetup_role: RoleId,
) {
    match event {
        Event::GatewayHeartbeatAck => (),
        Event::InteractionCreate(interaction) => {
            handle_interaction(*interaction, &http, secret_channel, meetup_role).await;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
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
    Clear,
    Meetup(bool), // enable or disable
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
                return Ok(Self::Add { title });
            }
            "agenda" => return Ok(Self::Agenda),
            "clear" => return Ok(Self::Clear),
            "meetup" => {
                for option in data.options {
                    if let CommandDataOption::SubCommand { name, .. } = option {
                        match name.as_str() {
                            "enable" => return Ok(Self::Meetup(true)),
                            "disable" => return Ok(Self::Meetup(false)),
                            _ => (),
                        }
                    }
                }
            }
            _ => (),
        }
        Err(())
    }
}

async fn handle_interaction(interaction: InteractionCreate, http: &HttpClient, secret_channel: ChannelId, meetup_role: RoleId) {
    match interaction.0 {
        Interaction::Ping(_) => println!("pong (interaction)"),
        Interaction::ApplicationCommand(application_command) => {
            let ApplicationCommand {
                channel_id,
                data,
                id,
                member,
                token,
                guild_id,
                ..
            } = *application_command;
            let response = if channel_id != secret_channel {
                "Commands are not valid in this channel".to_string()
            } else {
                match data.try_into() {
                    Ok(InteractionCommand::Add { title }) => {
                        Agenda::push_write(AgendaPoint {
                            title: title.to_string(),
                            adder: member.and_then(|m| m.nick).unwrap_or_else(|| "?".to_string()),
                            timestamp: chrono::Local::now(),
                        });
                        format!("Added {}", title)
                    }
                    Ok(InteractionCommand::Agenda) => {
                        get_agenda_string()
                    }
                    Ok(InteractionCommand::Clear) => {
                        let prev = get_agenda_string();
                        Agenda::clear();
                        format!("Previous agenda was:\n{}", prev)
                    }
                    Ok(InteractionCommand::Meetup(enable)) => {
                        if let Some(member) = member {
                            let has_meetup_role = member.roles.iter().any(|role| role == &meetup_role);
                            if enable && has_meetup_role {
                                "You already have this role".to_string()
                            } else if !enable && !has_meetup_role {
                                "You don't have this role".to_string()
                            } else if enable {
                                http
                                    .add_guild_member_role(
                                        guild_id.unwrap(),
                                        member.user.unwrap().id,
                                        meetup_role,
                                    )
                                    .reason("Requested by user")
                                    .unwrap()
                                    .exec()
                                    .await
                                    .unwrap();
                                "ok".to_string()
                            } else {
                                http
                                    .remove_guild_member_role(
                                        guild_id.unwrap(),
                                        member.user.unwrap().id,
                                        meetup_role,
                                    )
                                    .reason("Requested by user")
                                    .unwrap()
                                    .exec()
                                    .await
                                    .unwrap();
                                "ok".to_string()
                            }
                        } else {
                            "Missing member".to_string()
                        }
                    }
                    Err(_) => "Error parsing command".to_string(),
                }
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
            "{}",
            points
                .iter()
                .enumerate()
                .map(|(i, point)| format!("{}. {}", i, point))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}
