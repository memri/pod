# About
This documentation is part of [Pod](../README.md).

Plugins (previously also known as integrators)
are various "external" components that can enrich your data,
help you import your data from external services, push some data outside if you want, etc.

This page explains how Pod runs plugins.

# How to trigger
Plugins are automatically started when Pod certain items are requested to be created in Pod.
This is done via [HTTP API](./HTTP_API.md).

You can also try it out locally, see section below

# Manually trigger a plugin via Pod API
During development, you can use the following script to make Pod start a new Plugin (container)
```sh
owner="$SRANDOM$SRANDOM$SRANDOM"  # replace with desired owner, or leave as-is for tests
dbkey=  # note that the Plugin will not have access to this key, it'll only have `POD_AUTH_JSON`
container="test"

data=$(cat <<-END
{
    "auth": {
        "type": "ClientAuth",
        "databaseKey": "$dbkey"
    },
    "payload": {
        "createItems": [
            {"type": "Person", "id": "38583224e56e6d2385d36e05af9caa5e"},
            {"type": "StartPlugin", "container": "$container", "targetItemId": "38583224e56e6d2385d36e05af9caa5e"}
        ],
        "updateItems": [],
        "deleteItems": []
    }
}
END
)

curl -X POST -H "Content-Type: application/json" --insecure "http://localhost:3030/v3/$owner/bulk" -d "$data"
```

# Manually trigger a plugin from the command line
TL&DR; please use other test methods for simplicity.
However, if you need to know how it works, you can read below.

As described in [HTTP API](./HTTP_API.md), there are two authorization keys that a Plugin can use.

* `ClientAuth`, which requires knowing the `database_key`
* `PluginAuth`, which is how Pod really starts the Plugins

For Plugins that means:

* `ClientAuth` will never be *really* passed to the Plugin, but during tests,
  you can just use this auth because you have access to the (fake/test) `database_key` anyway.
* `PluginAuth` will actually be passed to the Plugin in a real system,
  but it is impossible to emulate it because the Pod keeps relevant encryption keys in-memory,
  generates then on startup and intentionally loses them on restart (for security reasons).
  In short, you cannot emulate `PluginAuth`, you can only call `Pod` to generate this Auth for you.

So regardless of which auth you use, the script below will give you an idea
of the basic structure of the docker command:
```sh
container="test"
owner="357535074912050791323416133896"
target_item='{"type":"Person","id":"38583224e56e6d2385d36e05af9caa5e","dateCreated":1623241923508,"dateModified":1623241923508",dateServerModified":1623241923508,"deleted":false}'
trigger_item_id="05abe8e2ef2d0fb4992239944a71bde5"  # the id of the item that started the Plugin (the StartPlugin item)
your_auth_json='{???}'  # depends on whether you use test auth or real system auth
network="localhost"  # "localhost" on linux, "host.docker.internal" on Mac and Windows

docker run \
  --network=host \
  --env=POD_FULL_ADDRESS="http://$network:3030" \
  --env=POD_TARGET_ITEM="$target_item" \
  --env=POD_OWNER="$owner" \
  --env=POD_AUTH_JSON="$your_auth" \
  --rm \
  --name="$container-$trigger_item_id" \
  -- \
  "$container"
```

# How are plugins started
Plugins are started via **docker** (or a dedicated container in production environment).
Pod will set the following environment variables for plugins:

* `POD_FULL_ADDRESS` = the address of Pod to call back,
  e.g. `https://x.x.x.x:80` or `http://localhost:3030`.
  You can call the endpoints via a URL like `$POD_FULL_ADDRESS/version`.
* `POD_TARGET_ITEM` = the JSON of the item that the plugin needs to run against.
* `POD_OWNER` = Pod owner information (to be used for auth).
* `POD_AUTH_JSON` = Data used for plugin authorization, "secretbox" style 
  (you're not expected and should not be able to see what's inside as contents are encrypted).
  This JSON should be passed back to Pod the way it is, no parsing or change is required
  (see HTTP_API for details).
