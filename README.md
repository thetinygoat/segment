# Segment
<img align="right" src="https://www.segment.wtf/img/logo.svg" height="150px" alt="segment fox logo">

Segment is a *simple* and *fast* in-memory key-value database written in Rust.

### Features
- Simple to use and understand.
- Keys can be separated into multiple dynamic keyspaces.
- Keyspace level configuration.

### Status
Segment is under heavy development and is in pre-alpha stages.

### Why Segment?
Segment's goal to is to provide a simpler and more intuitive in-memory key-value solution. It has certain features that other solutions don't. Let's go over them one by one.

#### Keyspaces
Segment has a concept of keyspaces. You can think of keyspaces like tables in a relational database, except that keyspaces don't have any schema. When the segment server starts there are no keyspaces and you can create as many as you like. 

#### Evictors
Separating keys into keyspaces comes with a benefit, we can now have keyspace level configurations. Evictors are one such configuration.
There are two types of evicros in Segment, **expiring evictors** and **max memory evictors**.

##### Expiring Evictors
The expiring evictor is responsible for evicting expired keys which runs for every keyspaces.

##### Max Memory Evictors
The second type of evictor is max memory evictor, which is responsible for evicting keys when the server reaches the max memory specified in `segment.conf`.
Currently there are 3 max memory evictors:
- Nop - Stands for no-operation which doesn't evict any keys.
- Random - Evicts keys in a random order.
- LRU - Evicts keys in a LRU fashion.

There are plans to include even more evictors out of the box in future.

Max memory evitors can configured at a keyspace level, which means that you can have a keyspace that does not evict at all while some keyspaces evict. 
This is powerful becuase now you don't have to spin up a separate server just because you want to have a separate eviction policy.

#### Multithreaded
Segment is multithreaded, which means it uses locks which can be a deal breaker for some use cases. But It works for most use cases and that's what segment is aiming for.

#### Ease of Use
Segment aims to be easy to use and intuitive. One way we are aiming to solve this is by having only one way of doing things. There is only one command to insert data and one way to get it back, this helps reduce the stuff that a developer needs to remember.

One more thing that we are doing is using simple commands, for example let's take a look at the command to insert some data in a keyspace.

```shell
SET my_keyspace my_key my_value IF_NOT_EXISTS EXPIRE_AFTER 60000
```

This commnd tells the segment server to insert the key `my_key` with value `my_value` into the keyspace `my_keyspace` if the key does not exist already and expire the key after 1 minute (60000ms). 

The command reads like english and the intent is clear

A similar command in redis would look like this.

```
SET my_key my_value NX EX 60
```
If you are not familiar with redis you will not understand what is happeining here. and if you want to have your ttl in milliseconds there is another flag for that and I don't even remember what it is called and that's the point, to reduce the dev effort.


### Installation

> Currently Segment is only tested on macOS (because that's what I have access to), but it should not be a problem to run it on linux. If you come across any errors during installation please feel free to open an issue.

Segment is built using Rust, so you will need rust and it's toolchain installed on your system. To install rust you can follow the steps [here](https://rustup.rs/).

After installing you can follow the steps below to build and run segment from source.

1. Clone this repository.
2. `cd /path/to/cloned/repo`
3. `cargo build --release`
4. The final binary can be found in `./target/release`

### Running the sever
After building you will find the `segment` binary in the `./target/release` directory.

Segment requires `segment.conf` file to start. `segment.conf` is the config file that contains several server configurations.

If you have the `segment.conf` file in the same directory as the segment binary you can just run the binary however, if the config file is in some separate directory you can start the Segment server using the command below

```shell
segment --config=/path/to/segment.conf
```

If the server is started successfully you will see a log similar to this in your terminal.

```shell
2022-10-29T07:23:05.308471Z  INFO segment::server: server started on port 1698
```

### List of Commands

#### `CREATE`
##### Description
Used to create a new keyspace. By defualt it doesn't take any arguments except the name of the keyspace, but you can specify the evictor you want to use for the keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace that you want to create.

##### Optional Arguments
- `EVICTOR` - Indicates the evictor that you want to use for the keyspace. Possible values include `NOP`, `RANDOM` and `LRU`.

##### Optional Flags
- `IF_NOT_EXISTS` - If a keyspace already exists and you try to create it again the server will throw an error, but if you don't want an error you can pass this flag with the create command.

##### Return Type
The return type can be a boolean or an error.

##### Examples
```shell
CREATE my_keyspace
```

```shell
CREATE my_keyspace EVICTOR LRU
```

```shell
CREATE my_keyspace EVICTOR LRU IF_NOT_EXISTS
```

#### `DROP`
##### Description
Used to drop a keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace that you want to drop.

##### Optional Flags
- `IF_EXISTS` - If a keyspace doesn't already exists and you try to drop it the server will throw an error, but if you don't want an error you can pass this flag with the drop command.

##### Return Type
The return type can be a boolean or an error.

##### Examples
```shell
DROP my_keyspace
```

```shell
DROP my_keyspace IF_EXISTS
```

#### `SET`
##### Description
Used to insert a value in the keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace that you want to create.
 - `<KEY>` - Key that you want to insert.
 - `<VALUE>` - Value for the key.

##### Optional Arguments
- `EXPIRE_AFTER` - Expiry time of the key in milliseconds after which it will expire.
- `EXPIRE_AT` - Unix timestamp after which the key will expire.

##### Optional Flags
- `IF_NOT_EXISTS` - If you want to set a key only if it does not already exists.
- `IF_EXISTS` - If you want to set a key only if it already exists.

##### Return Type
The return type can be a boolean or an error.

##### Examples
```shell
SET my_keyspace my_key my_value
```

```shell
SET my_keyspace my_key my_value IF_NOT_EXISTS
```

```shell
SET my_keyspace my_key my_value IF_EXISTS
```

```shell
SET my_keyspace my_key my_value EXPIRE_AFTER 60000
```

```shell
SET my_keyspace my_key my_value EXPIRE_AT 1667041052
```

#### `GET`
##### Description
Used to get a key from the keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace that you want to get the key from.
 - `<KEY>` - key that you want to get.


##### Return Type
The return type can be a string, null, or error.

##### Examples
```shell
GET my_keyspace my_key
```

#### `DEL`
##### Description
Used to delete a key from the keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace that you want to create.
 - `<KEY>` - Name of the keyspace that you want to create.


##### Return Type
The return type can be a boolean or error.

##### Examples
```shell
DEL my_keyspace my_key
```

#### `COUNT`
##### Description
Returns the number of keys in a keyspace.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace.


##### Return Type
The return type can be an integer or error.

##### Examples
```shell
COUNT my_keyspace
```

#### `TTL`
##### Description
Returns the remaining TTL of the key in milliseconds.

##### Essential Arguments
 - `<KEYSPACE>` - Name of the keyspace.
 - `<KEY>` - Name of the key.


##### Return Type
The return type can be an integer or null (if the key doesn't have an expiry or is already expired) or an error.

##### Examples
```shell
TTL my_keyspace my_key
```

#### `PING`
##### Description
Used to ping the server.


##### Return Type
The return type is the string `pong`.

##### Examples
```shell
PING
```

#### `KEYSPACES`
##### Description
Returns the list of keyspaces.

##### Return Type
The return type is an array of strings.

##### Examples
```shell
KEYSPACES
```



### Client Libraries and Utilities
Currenly there is a [Rust client](https://github.com/segment-dev/segment-rs) in very early stages. It is usable and works well, but not does not have a good DX and is missing several imporant features.

There is also a repo that aims to provide a collection of utilities for the Segment server like a CLI, a banchmarking tool etc. Currently it only contains a CLI which works well and can be used to play around with the server.

### Roadmap
The near term roadmap for Segment is to have tests in place (both unit and integration) so that it can be made produciton ready.

In the longer term, I would want to add some kind of persistance, and maybe make the server distributed :)
