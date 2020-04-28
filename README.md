# GameServer_Rust
> wrote the game server entirely in rust, and the design was inspired by the previous Java version of the server,
It contains several important components, as follows:

# # 1. gameserver:
> is dedicated to handling player data and logic, which already contains all the components required for the basic game suit, as follows (most of the following components are from tools) :<br>
>> a: TCP server is used to listen for messages sent by gateserver, based on mio(0.7.0) to achieve <br>
B: integrated mysql client, based on mysql(16.1.0)<br>
C :redis client, based on redis(0.13.0)<br>
D: the ThreadPool to be processed, using either async-std(1.5.0) or ThreadPool(1.7.1)<br>
E: asynchronous timer, used to perform some timed tasks, such as 0 point resetting daily, and persist player data to mysql server <br> asynchronously every 5 minutes
F: the HTTP server and HTTP client, based on async-h1(1.0.2) and http-types(1.1.0), are used to handle request events for some other component, such as shutting down the server, or other requirements <br>
G: and the source code with simple logic processing, communication protocol using TCP, based on mio(0.7.0) to achieve <br>
H: data storage adopts json. The framework selects serde and serde_json, which are designed for flexibility and convenience. < br >
I: log module, for logging and persistence to disk, divided into error.log and info.log<br>
# # 2. gateserver:
> is used to communicate with gameserver and roomserver. It is the bridge between gameserver and roomserver. Since it is stateless, does not cache any player data, and only USES it for message forwarding, there is no logical and persistent module for data processing
>> a: TCP server, used to listen for messages sent by game clients <br>
B: TCP client, divided into gameserver's TCP client and roomserver's TCP client, is used to send messages to the game client <br>
C: log module, used for logging and persistence to disk, divided into error.log and info.log<br>
D :websocket server, used to listen to the messages sent by the game client, d and a can be switched according to the needs, which you want to use is entirely up to you <br>
# # 3. roomserver:
> is used to handle combat related tasks and can be extended as required. The components are as follows (most of the following components are from tools) : <br>
A: TCP server, which listens for messages sent by gateserver <br>
B: reids client < br >
# # 4.tools:
> other projects of lib, encapsulate some TCP server, client, the purpose is to encapsulate the underlying components in a simple and easy to use, and then exposed the simple API for other projects to use. Other project TCP, HTTP, log, threadpool,protobuf files, etc., all come from it. Important components <br>
>> a: TCP, encapsulate TCP module, including client and server, expose API for upper layer to use <br>
B: HTTP, encapsulating the HTTP module, including the client and the server, exposes the API to the upper layer using <br>
C :log, encapsulate the log module, expose the API to the upper layer using <br>
D :protos, generates a good protobuf file that is provided to all projects using <br>
E :threadpool, encapsulates the threadpool module and exposes the API to the upper layer using <br>
F :cmd_code, encapsulated gameserver, roomserver, gateserver need to be responsible for the command segment <br>
G :conf, encapsulate the load json configuration file, expose the API to the upper use, such as mysql connection address, TCP listening address, redis connection address, and so on. < br >
H :template, encapsulate the load json configuration file, expose the API for the upper layer to use, such as load NPC configuration, map configuration, level configuration? It depends on you <br>
I :binary, which provides some apis for bit operations, such as two u32s into one u64, and one u64 into two u32<br>
J :util, provides some other, such as packet (message package), bytebuf (parse message package) <br>
## 5.net_test
> Various test codes
