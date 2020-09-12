# About
This documentation is part of [Pod](../README.md).

Data Collectives are a storage for data shared across different people. For example:

* communities, e.g. people interested in plants, food, etc
* family
* data that you can contribute to help building community Machine Learning tools ("datasets")
* teams in companies
* wikipedia-like articles


# Implementation

Data Collectives are implemented as a Pod variation with only specific API endpoints enabled.
Currently, only endpoints `insert_tree` and `version` are enabled.

All information stored by a Data Collective is stored in a single database,
and in order to write to a Data Collective, you need to know its `database_key`.
Reading data from a Data Collective via API is always impossible.

Data Collective maintainer must access the database from the filesystem,
presuming they also have the `database_key` of course.

Run Pod with `--help` to see CLI help on setting up a Data Collective.


# Front-ends

In order for front-ends to send information to Data Collectives,
they need to support configuration of the data collectives.

The information required is:

* `database_key`, which must be filled in by the user as it is a shared secret for all Data Collective participants
* URL of the Data Collective (similar to the one of Pod itself)
