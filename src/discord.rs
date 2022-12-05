use std::convert::{TryFrom, TryInto};

use color_eyre::eyre::{anyhow, bail};
use futures_util::stream::StreamExt;
use tokio::{
    join,
    sync::{broadcast, mpsc},
};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::{request::AuditLogReason, Client as HttpClient};
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{
            application_command::{CommandData, CommandDataOption, CommandOptionValue},
            ApplicationCommand, Interaction,
        },
    },
    gateway::payload::incoming::InteractionCreate,
    gateway::Intents,
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
};

use crate::{
    agenda::{Agenda, AgendaPoint},
    calendar::{self, model::Timestamp},
    kodapa, GenericRange,
};

pub async fn handle(
    token: String,
    _agenda_sender: mpsc::UnboundedSender<AgendaPoint>,
    event_receiver: broadcast::Receiver<kodapa::Event>,
) {
    let http = Box::new(HttpClient::new(token.clone()));
    let http = Box::leak(http) as &HttpClient;
    let secret_channel: Id<ChannelMarker> = Id::new(
        std::env::var("DISCORD_SECRET_CHANNEL")
            .expect("missing DISCORD_SECRET_CHANNEL")
            .parse()
            .unwrap(),
    );
    let meetup_role: Id<RoleMarker> = Id::new(
        std::env::var("DISCORD_MEETUP_ROLE_ID")
            .expect("missing DISCORD_MEETUP_ROLE_ID")
            .parse()
            .unwrap(),
    );

    let _e1 = join!(
        handle_discord_events(token, http, secret_channel, meetup_role),
        handle_reminder_events(event_receiver, http, secret_channel),
    );
}

async fn handle_reminder_events(
    mut receiver: broadcast::Receiver<kodapa::Event>,
    http: &HttpClient,
    secret_channel: Id<ChannelMarker>,
) {
    while let Ok(event) = receiver.recv().await {
        match event {
            kodapa::Event::Reminder { event } => {
                http.create_message(secret_channel)
                    .content(&get_meeting_string(&event))
                    .unwrap()
                    .exec()
                    .await
                    .unwrap();
            }
        }
    }
}

async fn handle_discord_events(
    token: String,
    http: &'static HttpClient,
    secret_channel: Id<ChannelMarker>,
    meetup_role: Id<RoleMarker>,
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
    // let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster.up().await;
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

        tokio::spawn(handle_event(
            shard_id,
            event,
            http,
            secret_channel,
            meetup_role,
        ));
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: &HttpClient,
    secret_channel: Id<ChannelMarker>,
    meetup_role: Id<RoleMarker>,
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

fn find_option<'a>(
    search_name: &str,
    iter: impl IntoIterator<Item = &'a CommandDataOption>,
) -> Option<&'a str> {
    iter.into_iter().find_map(|option| {
        let CommandDataOption { name, value, .. } = option;
        if let CommandOptionValue::String(value) = value {
            if name == search_name {
                return Some(value.as_str());
            }
        }
        None
    })
}

/// The kinds of interactions we support. Should match the file
/// `register_commands.py` which needs to be run if any changes to commands is
/// to be made.
//TODO: Use hyper instead of Python to register.
enum InteractionCommand {
    Add { title: String },
    Agenda,
    Meetup(bool), // enable or disable
    RemoveOne(usize),
    RemoveMany(Option<usize>, Option<usize>),
}

impl TryFrom<CommandData> for InteractionCommand {
    type Error = color_eyre::Report;

    fn try_from(data: CommandData) -> Result<Self, Self::Error> {
        match data.name.as_str() {
            "add" => {
                let title = find_option("title", data.options.iter())
                    .ok_or_else(|| anyhow!("no title"))?
                    .to_string();
                Ok(Self::Add { title })
            }
            "agenda" => Ok(Self::Agenda),
            "meetup" => {
                for option in data.options {
                    let CommandDataOption { name, .. } = option;
                    match name.as_str() {
                        "enable" => return Ok(Self::Meetup(true)),
                        "disable" => return Ok(Self::Meetup(false)),
                        _ => (),
                    }
                }
                todo!()
            }
            "remove" => {
                let which = find_option("which", data.options.iter())
                    .unwrap()
                    .to_string();

                if which.contains('-') {
                    let parts = which.split_once('-').unwrap();
                    let lower = Some(parts.0.parse::<usize>().expect("not a number") - 1);
                    let upper = Some(parts.1.parse::<usize>().expect("not a number") - 1);

                    Ok(Self::RemoveMany(lower, upper))
                } else {
                    Ok(Self::RemoveOne(
                        which.parse::<usize>().expect("not a number") - 1,
                    ))
                }
            }
            _ => bail!("unknown command {}", data.name.as_str()),
        }
    }
}

async fn handle_interaction(
    interaction: InteractionCreate,
    http: &HttpClient,
    secret_channel: Id<ChannelMarker>,
    meetup_role: Id<RoleMarker>,
) {
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
                            adder: member
                                .and_then(|m| m.nick.or(m.user.map(|user| user.name)))
                                .unwrap_or_else(|| "?".to_string()),
                            timestamp: chrono::Local::now(),
                        });
                        format!("Added {}", title)
                    }
                    Ok(InteractionCommand::Agenda) => get_agenda_string(),
                    Ok(InteractionCommand::RemoveOne(n)) => {
                        let prev = get_agenda_string();
                        Agenda::remove_one(n);
                        format!("Previous agenda was:\n{}", prev)
                    }
                    Ok(InteractionCommand::RemoveMany(lower, upper)) => {
                        let prev = get_agenda_string();
                        Agenda::remove_many(GenericRange(lower, upper));
                        format!("Previous agenda was:\n{}", prev)
                    }
                    Ok(InteractionCommand::Meetup(enable)) => {
                        if let Some(member) = member {
                            let has_meetup_role =
                                member.roles.iter().any(|role| role == &meetup_role);
                            if enable && has_meetup_role {
                                "You already have this role".to_string()
                            } else if !enable && !has_meetup_role {
                                "You don't have this role".to_string()
                            } else if enable {
                                http.add_guild_member_role(
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
                                http.remove_guild_member_role(
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
            let application_id = http
                .current_user_application()
                .exec()
                .await
                .unwrap()
                .model()
                .await
                .unwrap()
                .id;
            http.interaction(application_id)
                .interaction_callback(
                    id,
                    &token,
                    &InteractionResponse::ChannelMessageWithSource(CallbackData {
                        allowed_mentions: None,
                        components: None,
                        content: Some(response),
                        embeds: Default::default(),
                        flags: None,
                        tts: None,
                    }),
                )
                .exec()
                .await
                .unwrap();
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
                .map(|(i, point)| format!("{}. {}", i + 1, point))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}
