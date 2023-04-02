# Rust part

This folder represent an implementation of a dht using rust. 

## Key concepts
* We will try to do the same thing as the python part, but this time for real.
* I aimed at having something functionnal over something performant / efficient
* testing can be done with the script. Initial goal was to try to make it work
with docker swarm, but i couldnt wrap my head around how it works, and we don't really 
need the overhead that docker would provide 

## Requirement 
* a rust compiler 
* rights for port 6060-7979 (for testing, you can choose your own ports)
* tmux (optionnal, for testing)

### installation 
Use `cargo build` at the root of this folder.

### Testing
there is a small script, `./tmux-script.sh` to test things nicely. It runs inside tmux.  
(Note to teachers: if you run it from within the university's server, (gpu) i have 
installed tmux inside my session and chmod'ed it to 755, so you can use it :) 
it has been taken into account inside the testing script)  
### Usage
To run it, you'll need either 1 or 3 arguments : 
* <--bind-port> port to use for our machine
* [--remote-port] port to use for remote machine
* [--remote-ip] ip of remote machine

### Explanations
This program has two threads: one that listen for connection, and one that handle messages.
For now (WIP) we can only add and remove nodes from the ring. 

There are multiple mechanism to make this work:
* Initial connect(INIT CONNECT): 
Sends a connection request to a node. This should only be used as the first message a node 
sends. Upon reception of such message, the recipient will check if the nodes belongs to its sides, 
and act accordingly (insert directly, or send an insertion request to its neighbour).
* Request insertion(REQINS): This is sent when a node doesnt belong here. Upon reception, we check
if node belong here, and act accordingly.
* JOIN request(JOIN): This is to force a node to change its neighbours. This will assume the node 
belongs here. It will send its and its neighbours' info to the new node, and update its 
old neighbour on the new node.

Other than that, the process is pretty similar to the python simulation.

To resume, the general case goes like this: 
* a node wants to joins, it sends an INIT CONNECT to an entry point
* entry points checks if new nodes should replace one of its neighbour
	* if it does: replace node, sends REQINS containing new nodes' info to old 
	neighbour and JOIN to new node
	* if not: tell its neighbour to check (REQINS).

Cases to check, and solution: 
* end of ring (right neighbour hash < our hash or left neighbour hash > our hash)
	* we reached the end, and its not enough: insert here
* first join: do we insert left or right ?
	* we insert both, as left should also be right. We have to, otherwise our ring will be "open"
* join on yourself
	* That shouldnt happen, or we had a serious issue that i cant explain. Then, we crash and 
	try to leave the ring (peacefully(TODO)).

##TODOs / possible upgrade
* Because of the way we implemented neighbours, creating a leafset instead of having
only 2 neighbours would be somewhat easy to implement.
* sends file and store them according to their hash 
	* i didnt implement that since i was restarting the ring very often, and i thought it would
	be tedious to always resend the files. It probably won't, but i was too focused on making the 
	base work, wich isn't inherantly a bad thing.
* implement the dht struct in multiple files, its ugly as it is
	* didn't do that, since i didn't really take time to think how to divise it nicely, 
	so for now it's only one big mess of a file.
* Better logs ! 
	* Color output ! idk how to do that, but i didn't look far
	* one node only for logging
* i tried to make ctrl c exit nicely, but it blocks (sockets are sync and blocking)
	* make socket async (but last time it didnt go well, so i spent time where it was useful)
	so for now uppon a ctrl c it still operates normal, and quits uppon recieving (and handling)
	the next message.
