# MShot

You have multiple computers connected to the same network, and you want to take a screenshot of one or all of them? MShot is the solution for you!

_This is a WIP project, so it's not ready for production._

## Installation

### Server

Deploy the server on all computers that you want to take screenshots from.

```bash
# Run the server
$ mshot_server 127.0.0.1:1337
```

### Client

Install the client on the computer which you operate.

Assuming the network looks like this:
- Client: 192.168.1
- Server1: 192.168.11
- Server2: 192.168.21

```bash
# Take screenshot of Server1
$ mshot_client server1:http://192.168.11:1337

# Take screenshot of Server1 and Server2
$ mshot_client server1:http://192.168.11:1337 server2:http://192.168.21:1337
```

### Screenshots
Screenshots will be collectively saved into a new folder named after the current date and time, while the screenshots themselves will be named after the server's name.