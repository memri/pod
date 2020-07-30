## Action API - WhatsApp Functions

This guide focuses on how the WhatsApp-related functions provided by Action API can be used. 

The Action API can be accessed via the following URL: **POST /v2/$owner_key/do_action**  

To access Pod's API, a 

Each function is identified by client specifying the `actionType` in the JSON request when calling it. 
We give an example of a client request when calling one function and the corresponding response. 

### Accounts

Before you can send and receive messages via Action API, you must **register** for a Matrix account. If you already have an account, you must **login** into it.

#### Matrix registration

The aim of function `matrix_register` is to get a user ID and access token which you will need when accessing other functions.

Request example:
```shell script
{
    "actionType": "matrix_register",
    "content": {
        "messageBody": {
            "auth": {"type":"m.login.dummy"},
            "username": "foo",
            "password": "bar"
        }
    }	
}
```

Response example:
```shell script
{
    "access_token": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc", 
    "home_server": "my.homeserver", 
    "user_id": "@foo:my.homeserver"
}
```

#### Matrix login

The aim of function `matrix_login` is to get an access token for your existing user ID.

Request example:
```shell script
{
    "actionType": "matrix_login",
    "content": {
        "messageBody": {
            "type": "m.login.password",
            "user": "foo",
            "password": "bar"
        }
    }	
}
```

Response example:
```shell script
{
    "access_token": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc", 
    "home_server": "my.homeserver", 
    "user_id": "@foo:my.homeserver"
}
```

### Communication

In order to communicate via the Action API, you must **create a room** with that user and **send a message** to that room. 

#### Create a room

The aim of function `create_room` is to create a room for sending messages.

Request example:
```shell script
{
    "actionType": "create_room",
    "content": {
        "accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc",
        "messageBody": {
            "preset": "private_chat",
            "name": "whatsapp"
        }
    }	
}
```

Response example:
```shell script
{
  "room_id": "!tfhnHHvYFlZDbadcRA:my.homeserver"
}
```

#### Send messages

You can now use function `send_messages` to send a message.

Request example:
```shell script
{
	"actionType": "send_messages",
	"content": {
		"roomId": "!tfhnHHvYFlZDbadcRA:my.homeserver",
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc",
		"messageBody": {
			"msgtype": "m.text",
			"body": "login"
		}
	}
}
```

Response example:
```shell script
{
  "event_id": "$5gZNW1_Zzb_6iw4MkCsaQS9Wg0GwKNaQbyDo_iT8og8"
}
```

### Users and rooms

You can get which rooms you **have joined**, and **invite** others to join a room. You can also get the **members** who have joined.

#### Get joined rooms

The aim of function `get_joined_rooms` is to get a list of rooms the user has joined.

Request example:
```shell script
{
	"actionType": "get_joined_rooms",
	"content": {
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc"
	}
}
```

Response example:
```shell script
{
  "joined_rooms": [
    "!tfhnHHvYFlZDbadcRA:my.homeserver"
  ]
}
```

#### Invite a user to a room

Use function `invite_user_to_join` to directly invite a user to a room.

Request example:
```shell script
{
	"actionType": "invite_user_to_join",
	"content": {
		"roomId": "!tfhnHHvYFlZDbadcRA:my.homeserver",
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc",
		"messageBody": {
			"user_id": "@whatsappbot:my.homeserver"
		}
	}
}
``` 

Response example:
```shell script
{}
```

#### Get joined members of a room

Use function `get_joined_members` to list all members that have joined a room.

Request example:
```shell script
{
	"actionType": "get_joined_members",
	"content": {
		"roomId": "!tfhnHHvYFlZDbadcRA:my.homeserver",
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc"
	}
}
```

Response example:
```shell script
{
  "joined": {
    "@foo:my.homeserver": {
      "avatar_url": null,
      "display_name": "foo"
    },
    "@whatsappbot:my.homeserver": {
      "avatar_url": "mxc://maunium.net/NeXNQarUbrlYBiPCpprYsRqr",
      "display_name": "WhatsApp bridge bot"
    }
  }
}
```

### Get events

An event is a piece of data, can be a message in a room, a room invitation, etc. There are different ways of getting events, depending on what the client already knows, e.g. by **synchronizing** all rooms, or **getting room messages**.

#### Sync live states

Use function `sync_events` to get the updates of all users' information of all rooms you have joined since the point under the object key `next_batch`. This key is included in the response. 

Request example:
```shell script
{
	"actionType": "sync_events",
	"content": {
		"nextBatch": "s9_7_0_1_1_1",
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc"
	}
}
```  

Response example:
```shell script
{
    "account_data": {
        "events": []
    },
    "next_batch": "s9_9_0_1_1_1",
    "presence": {
        "events": [
            {
                "content": {
                    "currently_active": true,
                    "last_active_ago": 12,
                    "presence": "online"
                },
                "sender": "@foo:my.homeserver",
                "type": "m.presence"
            }
        ]
    },
    "rooms": {
        "invite": {},
        "join": {},
        "leave": {}
    }
}
```

#### Get events of a room

If you know which room you have joined, you can use function `get_messages` to get events only from that room.

Request example:
```shell script
{
	"actionType": "get_messages",
	"content": {
		"nextBatch": "s9_7_0_1_1_1",
		"roomId": "!tfhnHHvYFlZDbadcRA:my.homeserver",
		"accessToken": "QGV4YW1wbGU6bG9jYWxob3N0AqdSzFmFYrLrTmteXc"
	}
}
```

Response example:
```shell script
{
  "chunk": [
    {
      "age": 29734,
      "content": {
        "body": "login",
        "msgtype": "m.text"
      },
      "event_id": "$5gZNW1_Zzb_6iw4MkCsaQS9Wg0GwKNaQbyDo_iT8og8",
      "origin_server_ts": 1596122404778,
      "room_id": "!InoezKbWDDEPNphKlq:bli-ws",
      "sender": "@bli:bli-ws",
      "type": "m.room.message",
      "unsigned": {
        "age": 29734
      },
      "user_id": "@bli:bli-ws"
    }
  ],
  "end": "t21-39_7_0_1_1_1_0_0_0",
  "start": "s19_7_0_1_1_1_0_0_0"
}
```
