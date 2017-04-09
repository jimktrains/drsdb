# drsdb - Distributed, Rust-based SQL DataBase

The basic concept is that of a three-tiered system:

* Client
* Index Server
* Storage Server

Each tier will have specific responsibilities and properties. Each is
crucial to some part of query execution. Combined they form a resilient
and load balanced system.

The Client compiles queries, sends them off,and waits for a response. The
Client will select a single index server (e.g. DNS Round Robin) to
communicate with for this query.  If the server doesn't respond, the next
server is tried. Upon two failures, the server is removed from the rotation
and an updated list of index servers should be retrieved.

The Index Server performs as much filtering and joining as possible. The
Index Server could additionally perform caching. The Index Servers are also
responsible for distributing writes to nodes such that data is redundantly
stored across the server pool.

The Storage Servers store and retrieve data from disk. They may cache data
in memory.  For joined data, they compute the final rows and perform
filtering. (This may require a request to other servers.)  For aggregate
queries, they aggregate the rows available to them and then send or receive
the partial aggregates to form the complete aggregate. Servers then send
results directly to clients

                  +--------------+----->+----------------+
                  | Index Server |   +->| Storage Server |
                  +--------------+--+|  +----------------+
                                    ||     |
    +--------+<----------------------------+
    | Client |                      ||
    +--------+--->+--------------+  +-->+-----------------+
                  | Index Server |---|  | Storage Server  |
                  +--------------+----->+-----------------+

This project will start with a subset of
[SQL 92](https://www.contrib.andrew.cmu.edu/~shadow/sql/sql1992.txt),
as the spec can be found freely online.

Joins will be slow, but should be supported. I would like to explore
automatically indexing foreign key constraints in order to figure out
how to better store data to enable joins to be more performant.

# Timeline
## (V 0.1) Simple-on-disk format R/W

I'm debating between backing with SQLite, or perhaps (G)DBM.  I would
like the ability for these to be able to support ACID writes.

N.B.: The Storage Server will be able to use heterogeneous storage methods
(either by server or by table, I haven't thought about it). This is just the
first, and probably simplest row format that will be supported. I'd imagine a
columnar format in the future as well.


## (V 0.2) Index support

A database that doesn't support indices, while possible (viz. BigQuery)
isn't what we're after here.

## (V 0.3) Lexical-Distributed  IDs

Instead of sequential ids, Snowflake ([[0]](https://web.archive.org/web/20101006173631/http://github.com/twitter/snowflake)
[[1]](http://github.com/twitter/snowflake) [[2]](http://rob.conery.io/2014/05/29/a-better-id-generator-for-postgresql/))
or [ULID](https://github.com/alizain/ulid).  Having to consult a distributed
log an extra time during creation isn't all that nice to do.

## (V 0.4) SQL Parser (DQL Only)

We'll start by only parsing queries and running them.

## (V 0.5) DTLS Client - Server simple read/write

Secure and authenticated communication between servers and clients is a must
and should be the default. 0-RTT with TLS/1.3 will reduce overhead on
on systems that have been running for a while.

We'll be using protocol buffers as our data serialization medium.

## (V 0.6) Split Index and Storage nodes

Once we're able to query, the Index and Storage Servers should be split
from the single server.

## (V 0.8) SQL Parser (DML Only)

Being able to update tables is nice! This will require a consensus algorithm
such as [Raft](https://raft.github.io/) or a
[Total Order Broadcast algorithm](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.110.6701&rep=rep1&type=pdf)

## (V 0.7) SQL Parser (DDL Only)

Being able to create tables is nice.

