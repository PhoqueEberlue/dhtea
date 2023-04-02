#!/bin/bash 

if [[ -f "/home/paulinma/tools/tmux/usr/bin/tmux" ]]; then 
	TMUX=/home/paulinma/tools/tmux/usr/bin/tmux
else
	TMUX=$(whereis tmux)
fi

# create main entry point, or at least the first
echo "
██████╗ ██╗  ██╗████████╗███████╗ █████╗
██╔══██╗██║  ██║╚══██╔══╝██╔════╝██╔══██╗
██║  ██║███████║   ██║   █████╗  ███████║
██║  ██║██╔══██║   ██║   ██╔══╝  ██╔══██║
██████╔╝██║  ██║   ██║   ███████╗██║  ██║
╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝

rust edition"

echo using tmux from \'$TMUX\'
$TMUX new-session -s "dht" -d "./target/debug/dht --bind-port 6060"
$TMUX rename-window "dht testing"
echo "this is a demo to view how the project works, and to demonstrate that it does :)"
echo "to continue, please open another terminal with, and attach the $TMUX session 'dht'"
echo "('tmux a' should do the trick)"
echo "press return to continue..."
read
echo "We will go step by step for this one, for the purpose of seing if everything works."

echo "Next we will go all together, to check if a node can handle joining two nodes joining \
at about the same time"
echo "press return to continue..."
read

# connect a few nodes to it
$TMUX split-window -h "./target/debug/dht --bind-port 6161 --remote-ip 127.0.0.1 --remote-port 6060"
$TMUX select-layout even-horizontal
echo "Joined 6161 to 6060. press return to continue..."
read
$TMUX split-window -h "./target/debug/dht --bind-port 6262 --remote-ip 127.0.0.1 --remote-port 6060"
$TMUX select-layout even-horizontal
echo "Joined 6262 to 6060. press return to continue..."
read
$TMUX split-window -h "./target/debug/dht --bind-port 6363 --remote-ip 127.0.0.1 --remote-port 6060"
$TMUX select-layout even-horizontal
echo "Joined 6363 to 6060. press return to continue..."
read
$TMUX split-window -h "./target/debug/dht --bind-port 6464 --remote-ip 127.0.0.1 --remote-port 6060"
$TMUX select-layout even-horizontal
echo "Joined 6464 to 6060"
echo "Check if everything is ok ! next is joining to new-ish nodes."
echo "press return to continue..."
read 

# new window !
# connect other nodes to the ones connected in the previous step
$TMUX split-window -v "./target/debug/dht --bind-port 6565 --remote-ip 127.0.0.1 --remote-port 6363"
echo "Joined 6565 to 6363. press return to continue..."
read
$TMUX select-pane -L
$TMUX split-window -v "./target/debug/dht --bind-port 6666 --remote-ip 127.0.0.1 --remote-port 6464"
echo "Joined 6666 to 6464. press return to continue..."
read
$TMUX select-pane -L
$TMUX split-window -v "./target/debug/dht --bind-port 6767 --remote-ip 127.0.0.1 --remote-port 6161"
echo "Joined 6767 to 6161. press return to continue..."
read
$TMUX select-pane -L
$TMUX split-window -v "./target/debug/dht --bind-port 6868 --remote-ip 127.0.0.1 --remote-port 6262"
echo "Joined 6868 to 6262."
echo "Check if everything is ok ! next is joining multiple nodes to one node all at once."
echo "press return to continue..."
read 

$TMUX new-window "./target/debug/dht --bind-port 7070 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX split-window -v "./target/debug/dht --bind-port 7171 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX split-window -h "./target/debug/dht --bind-port 7272 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX split-window -v "./target/debug/dht --bind-port 7373 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX split-window "./target/debug/dht --bind-port 7474 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX split-window "./target/debug/dht --bind-port 7575 --remote-ip 127.0.0.1\
	--remote-port 6060"
$TMUX select-layout tiled

echo "those are in a new window, you can check previous by typing ctrl b + n"

echo "end of demo ! :) sorry couldnt learn docker in time, and thats the most readable way i found"
echo "press return to kill all sessions"
read
$TMUX kill-session -t dht
