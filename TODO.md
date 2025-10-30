# GENERAL INSTRUCTIONS

## DEMO
- [game like example](https://www.youtube.com/watch?v=5V5X5SbSjns)


## 3D

### 3D/ENGINE
- [bevy](https://bevy.org/)
- [install](https://bevy.org/learn/quick-start/getting-started/setup/)
- [tuto](https://www.youtube.com/watch?v=JxrGAGWDv6M)


### 3D/INSTRUCTIONS
- A mini map where the player can see his own position and the whole "game world".  
- The graphics of the game (walls and other players) should be similar to the original game (see maze_wars for more details)   
- Finally you have to display the frame rate of the game on the screen.  



## ARCHITECTURE

### ARCHITECTURE/SERVER
- Implement a client-server architecture where clients connect to a central server to play the game.
- Your implementation should allow one client and the server to run on the same machine, with other clients connecting from different machines.
- Use the UDP protocol to enable the communication between the clients and the server.
- The game should have at least 3 levels with increasing difficulty (with difficulty we mean, making the maze harder, with more dead ends).

You will have to develop the game server and also a client application:
- The server must accept as many connections as possible (the minimum should be 10).
- When the client is initialized, the game should ask for:
    - The IP address of the server, allowing the client application to connect to any server.
    - A username for identification.


### ARCHITECTURE/TERMINAL INTERFACE
- After providing the above information, the game should start and open the graphical interface, allowing the player to join and start playing the game.
- Example: Assuming that you can connect to a server in the same computer.
```sh
$ cargo run
Enter IP Address: 198.1.1.34:34254
Enter Name: name
Starting...
$
```

## PERFORMANCE
- The game should always have a frame rate above 50 fps (frames per second).
