-- Add migration script here
-- a single table is used for all events in the cqrs system
CREATE TABLE events
(
  aggregate_type text                          NOT NULL,
  aggregate_id   text                          NOT NULL,
  sequence       integer CHECK (sequence >= 0) NOT NULL,
  event_type     text                          NOT NULL,
  event_version  text                          NOT NULL,
  payload        text                          NOT NULL,
  metadata       text                          NOT NULL,
  PRIMARY KEY (aggregate_type, aggregate_id, sequence)
) STRICT;


CREATE TABLE deployment_query
(
  view_id text                         NOT NULL,
  version integer CHECK (version >= 0) NOT NULL,
  payload text                         NOT NULL,
  PRIMARY KEY (view_id)
) STRICT;