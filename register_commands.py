import config
import requests

# See https://discord.com/developers/docs/interactions/slash-commands#registering-a-command

url = f"https://discord.com/api/v8/applications/{config.DISCORD_APPLICATION_ID}/guilds/{config.DISCORD_GUILD_ID}/commands"

json = {
    "name": "add",
    "description": "Add a thing to the agenda",
    "options": [
        {
            "name": "title",
            "description": "What to add",
            "type": 3,
            "required": True,
        },
    ]
}

headers = {
    "Authorization": f"Bot {config.DISCORD_BOT_TOKEN}"
}

r = requests.post(url, headers=headers, json=json)
print(r)
