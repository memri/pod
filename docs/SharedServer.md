# About
This documentation is part of [Pod](../README.md).

A Shared Pod is a type of Pod that multiple people can write to. For example:

* communities, e.g. people interested in plants, food, etc
* family
* data that you can contribute to help building community Machine Learning tools ("datasets")
* teams in companies
* wikipedia-like articles
* etc


# Front-ends

In order for front-ends to send information to Shared Pods,
they need to support their configuration.

Each Shared Pod has:

* `database_key`, which must be filled in by the user as it is a shared secret for all Shared Pod participants
* The `owner` key of the Shared Pod.
* URL of the Shared Pod (similar to the one of Pod itself).

It is the front-end-s decision on which data to send to a particular Shared Pod.
It always needs to be done with user confirmation.


# Implementation

Shared Pods are currently implemented as a run mode of the Pod server.
The owner of the shared Pod hosts a public instance and gives
the `owner` key and the `database_key` to anyone they want.
Any person having this `owner` and `database_key` will then be able to
connect to the shared pod and write data to it.

This run mode is activated with the `--shared-server` CLI parameter, and it means that the
server will only have `create_item` and `version` (write-only) endpoints working.
Because it is write-only, other users won't be able to read your data,
and you can submit even your sensitive data if you trust the Shared Pod maintainer.

In the future we expect users to be able to share data with more fine-grained permissions,
e.g. by allowing reads but not edits, etc.

Shared Pod maintainer must access the database from the filesystem.
(To do so, they also need to have the `database_key` of course.)


# Debugging

If you want to experiment with how a shared Pod works,
run your Pod with the `--shared-server` CLI parameter.
For example, `./examples/run_development.sh --shared-server`.
