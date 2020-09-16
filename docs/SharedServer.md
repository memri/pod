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
* URL of the Shared Pod (similar to the one of Pod itself)

It is the front-end-s decision on which data to send to a particular Shared Pod.
It always needs to be done with user confirmation.


# Implementation

Shared Pods are implemented as a variation of Pod. Currently,
it is limited to `insert_tree` and `version`, which are basically write-only endpoints.
This makes sure that you can submit even your sensitive data if you trust the Shared Pod maintainer.

In the future we expect users to be able to share data with more fine-grained permissions,
e.g. by allowing reads but not edits, etc.

All information stored by a Shared Pod is stored in a single database,
and in order to write to a Shared Pod, you need to know its `database_key`.

Shared Pod maintainer must access the database from the filesystem.
(To do so, they also need to have the `database_key` of course.)

Run Pod with `--help` to see CLI help on setting up a Shared Pod.
