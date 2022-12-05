import config
import json
import requests

# See https://discord.com/developers/docs/interactions/slash-commands#registering-a-command

BASE_URL = "https://discord.com/api/v8"
HEADERS = {
    "Authorization": f"Bot {config.DISCORD_BOT_TOKEN}"
}
COMMANDS = {
    "add": {
        "name": "add",
        "description": "Add a thing to the agenda",
        "options": [
            {
                "name": "title",
                "description": "What to add",
                "type": 3,
                "required": True,
            },
        ],
        "default_permission": False,
    },
    "agenda": {
        "name": "agenda",
        "description": "List the current agenda",
        "options": [],
        "default_permission": False,
    },
    "remove": {
        "name": "remove",
        "description": "Remove one or more items from the agenda",
        "options": [
            {
                "name": "which",
                "description": "Which item(s) to remove",
                "type": 3,
                "required": True,
            }
        ]
    },
    "clear": {
        "name": "clear",
        "description": "Clear the current agenda",
        "options": [],
        "default_permission": False,
    },
    # "meetup": {
    #     "name": "meetup",
    #     "description": "Configure meetup notifications",
    #     "options": [
    #         {
    #             "name": "enable",
    #             "description": "Enable meetup notifications",
    #             "type": 1,
    #             "options": [],
    #         },
    #         {
    #             "name": "disable",
    #             "description": "Disable meetup notifications",
    #             "type": 1,
    #             "options": [],
    #         },
    #     ],
    # },
}


def update_command(command, data):
    url = f"{BASE_URL}/applications/{config.DISCORD_APPLICATION_ID}/guilds/{config.DISCORD_GUILD_ID}/commands"
    print("POST", command)
    resp = requests.post(url, headers=HEADERS, json=data)
    if resp.status_code != 200:
        print(json.dumps(resp.json(), indent=2))
    print(resp)


def fetch_commands():
    url = f"{BASE_URL}/applications/{config.DISCORD_APPLICATION_ID}/guilds/{config.DISCORD_GUILD_ID}/commands"
    resp = requests.get(url, headers=HEADERS)
    print(json.dumps(resp.json(), indent=2))


def update_commands(commands=None):
    if commands is not None:
        for command in commands:
            update_command(command, COMMANDS[command])
    else:
        for command, data in COMMANDS.items():
            update_command(command, data)


def allow_command_for_role(command_id, role_id):
    url = f"{BASE_URL}/applications/{config.DISCORD_APPLICATION_ID}/guilds/{config.DISCORD_GUILD_ID}/commands/{command_id}/permissions"
    data = {
        "permissions": [
            {
                "id": role_id,
                "type": 1,
                "permission": True,
            }
        ],
    }
    resp = requests.put(url, headers=HEADERS, json=data)
    if resp.status_code != 200:
        print(json.dumps(resp.json(), indent=2))
    print(resp)


def remove_command(id):
    url = f"{BASE_URL}/applications/{config.DISCORD_APPLICATION_ID}/guilds/{config.DISCORD_GUILD_ID}/commands/{id}"
    resp = requests.delete(url, headers=HEADERS)
    print(resp)
    if resp.status_code != 200:
        print(json.dumps(resp.json(), indent=2))


update_commands(["remove"])
fetch_commands()
