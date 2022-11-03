# Segment Wire Protocol
This document is the specification for the Segment wire protocol.

## Introduction
Segment has a text based TCP protocol that is used for client-server architecture. It is inspired by [RESP](https://redis.io/docs/reference/protocol-spec/) and [Memcached](https://github.com/memcached/memcached/blob/master/doc/protocol.txt). The protocol aims to be *human readable*, *fast to parse* and *simple to implement*.

## Request-Response Model
The client connects to the Segment server on port 1698 by default. The client sends a command which is made up of arguments and flags, on receiving a request the server sends back a reply. This is a traditional client-server architecture.

## Protocol Specification

### Data Types
The protocol supports 8 data types:

- [Segment Wire Protocol](#segment-wire-protocol)
  - [Introduction](#introduction)
  - [Request-Response Model](#request-response-model)
  - [Protocol Specification](#protocol-specification)
    - [Data Types](#data-types)
      - [Strings](#strings)
      - [Integers](#integers)
      - [Doubles](#doubles)
      - [Booleans](#booleans)
      - [Null](#null)
      - [Errors](#errors)
      - [Arrays](#arrays)
      - [Maps](#maps)

The first byte determines the data type.

- For **Strings** the first byte is `$`
- For **Integers** the first byte is `%`
- For **Doubles** the first byte is `.`
- For **Booleans** the first byte is `^`
- For **Null** the first byte is `-`
- For **Errors** the first byte is `!`
- For **Arrays** the first byte is `*`
- For **Maps** the first byte is `#`

The protocol uses CRLF (`\r\n`) as the delimiter.

#### Strings
Strings are encoded as follows: A `$` character followed by the length of the string followed by CRLF. After encoding the length, the actual data is appended followed by a CRLF.

```
$11\r\nhello world\r\n
```

Since the strings are prefixed with their lengths we don't need to search for any delimiter to mark the end of the string. This makes it fast to parse and it also makes the strings **binary safe**.

#### Integers
Integers are encoded as follows: A `%` character followed by the integer that we want to encode followed by CRLF.

```
%100\r\n
```

#### Doubles
Doubles are similar to integers, they are encoded as follows: A `.` character followed by the double that we want to encode followed by CRLF.

```
.26.3\r\n
```

#### Booleans
Booleans are encoded as follows: A `^` character followed by a `0` for false and a `1` for true followed by CRLF.

```
^0\r\n // false

^1\r\n // true
```

#### Null
Nulls are encoded as follows: A `-` character followed by CRLF.

```
-\r\n
```

#### Errors
Errors are similar to strings and are encoded as follows: A `!` character followed by the length of the error data followed by CRLF. After encoding the length, the data is appended followed by CRLF

```
!5\r\nerror\r\n
```

#### Arrays
Array is a container type, it can contain all the other data types. An array is encoded as follows: A `*` character followed by the number of items in the array followed By CRLF. After encoding the length of the array we can just encode any type into it. Arrays can contain different data types at once.

```
// empty array
*0\r\n

// array  containing one integer
*1\r\n%100\r\n 

// array containing an integer and a string
*2\r\n%100\r\n$5\r\nhello\r\n
```

#### Maps
A map is a hash map, it's similar to an array and is encoded as follows: A `#` character followed by the number of items in the map followed by CRLF. Please note that a key-value pair is considered as a single unit/item. After encoding the number of items we can encode any type as key and value. Even though a key can be of any type, segment will only send keys as strings.

```
// empty map
#0\r\n

// map containing one key-value pair, with key being hello and value being world
#1\r\n$5\r\nhello\r\n$5\r\nworld\r\n
```