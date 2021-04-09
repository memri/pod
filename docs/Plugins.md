# About
This documentation is part of [Pod](../README.md).

Plugins (previously also known as integrators)
are various "external" components that can enrich your data,
help you import your data from external services, push some data outside if you want, etc.

This page explains how Pod runs plugins.

# How to trigger
Plugins are automatically started when Pod certain items are requested to be created in Pod.
This is done via [HTTP API](./HTTP_API.md).

You can also manually trigger a plugin by just emulating the below steps

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
