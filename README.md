
# Jetbrains intership task solution
Science I really enjoy programiring in rust, I will use it to implement this project.

## Task

The task involves interacting with a glitchy HTTP server that sends randomized data in partial chunks. The server supports the HTTP "Range" header, allowing clients to request specific portions of data. The goal is to write a client application that successfully downloads the complete binary data despite the server's unreliable behavior. The downloaded data must be verified by comparing its SHA-256 hash with the hash displayed by the server.

Rather than simply implementing a basic client, I decided to try to design a maintainable, extensible system architecture. This approach allows for trying different data retrieval strategies, testing components in isolation, and adapting to potential future changes in server behavior. The system is built with clean(in my opinion) interfaces between components, making it easy to swap out different manager implementations for experimenting with various retrieval algorithms.

## Hex
Very simple implementation of hex encodoing from byte array.

## Interval List
During the task we literally works with chunks of random length, so I think there is no needed explation, why I choise this ds.

### Improvements:
1. Optimizations. For example merge operation, instead of adding each element of the list, we can skip chunks which are already included in the main list.

## HTTP Messages
In the crate I implemented a very simple http response and request strucutres, serialization and deserialization for them.

### Improvements:
1. Add support for more HTTP methods (GET, POST, PUT, DELETE)
2. Using rust type system make structures type safe. For example *Range* header could be not just a string, but a range from rust.

## Server Communicator
As current server is very simple, it would potentially improved a lot in the future. So I decided to create an api which would not deppend on the server.

The core of the application communicates with the communicator through mpsc channels, one channel for requests and another for responses.

As the send operation is very cheap and comletely non-blocking, the main application can send as many requests as needed, without loosing any perfomance, while the communicator part will handle perfomant requesting data from the server.

### Improvements:
1. The current implementaion is very simple, and does not cover any features of newer http versions.

## Manager
The manager is responsible for manage the current state of the data. It has 2 core function request and receive methods.
- Receive method handle incoming chunks (in current implementaion it also call the request method, but its unnecessary to do so, because )
- Request method send request chunk, an important part is that idealy response should be deppended only on the current state of the manager, not on the last received response. Due to this restriction, it would be easier to design more complex systems.

Currently I implemented 2 types of managers:
1. A *Simple manager* which request data from the current left bound until the max data len. and repeat it until the wholde data will be received.
2. A *Random manager* which request random chunks of unknown data. It took aprroximately 40 minutes to implement and test this manager using existing apis.


Becase the server is single threaded, I do not see better implementaion then the current one. But when server switch at least at HTTP1/1 and parallel requests processing will be available, it would become possible to take a profit from more complex managers.

## Manager Wrapper
Its very imporatnt to test managers, and it would be hard and unperfomant to test different managers with real the server. So I added a manager wrapper that allow to test managers with different data holders(server). I implemented faster copy of original  server, which generate a random chunk of data from the range without any timeouts.

Also its the place where you can change the programs workflow, create serialize and deserialize http messages, and in the end add some overhead logic.

In current implemenatin request manager's method is called on each receiver response, but it unnecessary to do so, for example you could implement some parallel system.

## Improvements
The implementaion is not dependend on complexity of the manager, but can be changed with increase of server responses sending rate.

## Difficulties
1. I had some problems in implementing interval list, because of my confusion in ranges, speccificly in the begging i included both left and right bounds, which was very uncomfortable to work with. So in the end left bound is included and right bound is excluded.

2. For some reasons I thought that the server returns "Content-Range" header, but api I designed allowed a way out, by hardcodoing the last requested range in manager wrapper (this solution will not work for more complex systems, where manager can request more then one chunk, before receiving the response).


## Conclusion
I really enjoy solving this assigment. I think I was able to design maintainable and adaptable system. For new algorithms creation and testing. But there are still a lot to work on:

- Optimizations
- Switching to next http versions
- More algrithms desing
- Adding some metrics and logging

This project demostrates my ability to design clean system, implement algorithms, and create testable and maintable code in rust.

## Run
I have played a bit with cargo scripts.

Script signature:
```bash
cargo run --release -- --hash your hsah in hex --addr server address --manager manager type
```

### Description:

With hash and addr arguments, its quite obvious: the hash of servers data, and its address.

The manager argument stands for the type of manager program uses (for new managers need to extend crate::arguments::get_manager_function and ManagerType enum)

(to see my logs just run in dev mode (without release feature))

You can also configure environment variables:

- HASH: hash of servers data
- ADDR: address of server
- MANAGER: type of manager program uses

### Tests:
If you want to run the application in tests mode, to test managers on server simulation, you can just

```bash
cargo test test_name -- --nocapture
```

you can get test_name from project/tests/mod.rs

use -- --nocapture if you want to see all logs.
