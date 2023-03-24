import simpy
import random
import networkx as nx
import matplotlib.pyplot as plt

# Colors
black = "\033[0;30m"
red = "\033[0;31m"
green = "\033[0;32m"
yellow = "\033[0;33m"
blue = "\033[0;34m"
magenta = "\033[0;35m"
cyan = "\033[0;36m"
white = "\033[0;37m"
bright_black = "\033[0;90m"
bright_red = "\033[0;91m"
bright_green = "\033[0;92m"
bright_yellow = "\033[0;93m"
bright_blue = "\033[0;94m"
bright_magenta = "\033[0;95m"
bright_cyan = "\033[0;96m"
bright_white = "\033[0;97m"


class Message(object):
    def __init__(self,
                 type: str,
                 from_node: 'Node',
                 to_node: 'Node',
                 node_to_connect: 'Node' = None,
                 neighbour: 'Node' = None):

        # Message types:
        # 'connect': connection request
        # 'change_neighbour': change of neighbour request
        # 'new_neighbour': new neighbour request

        self.type: str = type
        self.from_node: 'Node' = from_node
        self.to_node: 'Node' = to_node
        self.node_to_connect = node_to_connect

        self.neighbour = neighbour

    def __repr__(self):
        return f"node_to_connect: {self.node_to_connect} neighbour: {self.neighbour}"


class Node(object):
    def __init__(self, env, i, color, connected=False):
        self.env = env

        # Implementation
        self.left_node: Node = None
        self.right_node: Node = None
        self.id = i
        self.color = color
        self.message_queue: list[Message] = []
        self.connection_queue: list[Message] = []

        # Status variables
        self.connected = connected
        self.waiting_for_connection = False
        self.lock_connection_request = False

    def get_id(self):
        """returns the identifier of the node"""
        return self.id

    def send(self, message: Message):
        """Sends a message to another node after a random timeout"""
        if message.type == "connect":
            self.waiting_for_connection = True
            self.lock_connection_request = True

        print(f"[{self.env.now}][{message.type}] {message.from_node} -> {message.to_node}")

        yield self.env.timeout(random.randint(1, 10))
        message.to_node.receive(message)

    def receive(self, message: Message):
        """
        Receive a message from another node and appends it to the message queue
        """
        print(f"[{self.env.now}][{message.type}] {self} <- {message.from_node} content: {message}")
        self.message_queue.append(message)

    def process_messages(self):
        """Proccess every messages from the message queue"""
        if len(self.message_queue) != 0:
            for message in self.message_queue:
                if "connect" in message.type:
                    self.connection_queue.append(message)
                elif "change_neighbour" in message.type:
                    yield from self.manage_change_neighbour_request(message)
                elif "new_neighbour" in message.type:
                    yield from self.manage_new_neighbour(message)
                elif "success_join" in message.type:
                    self.lock_connection_request = False

            self.message_queue.clear()
        else:
            # Small wait if there are no message in the queue
            yield self.env.timeout(1)

        if not self.lock_connection_request and len(self.connection_queue) != 0:
            yield from self.manage_connection_request(
                    self.connection_queue.pop(0)
                    )
        else:
            if len(self.connection_queue) > 0:
                message = self.connection_queue[0]
                print(f"[{self.env.now}][CONNECTION QUEUE] {self} has {message.from_node} in queue")

    def manage_connection_request(self, message):
        self.lock_connection_request = True

        node_id = message.node_to_connect.get_id()

        redirect = False
        change = False
        right = None
        to_node = None

        # Checking redirection
        if node_id > self.right_node.get_id() and self.right_node.get_id() > self.get_id():
            redirect = True
            to_node = self.right_node
        elif node_id < self.left_node.get_id() and self.left_node.get_id() < self.get_id():
            redirect = True
            to_node = self.left_node

        # Checking which side the node should be added
        elif node_id > self.get_id():
            change = True
            to_node = self.right_node
            right = True
        elif node_id < self.get_id():
            change = True
            to_node = self.left_node
            right = False

        if redirect:
            # Redirecting the connection request
            yield from self.send(
                    Message("connect_redirect",
                            from_node=self,
                            to_node=to_node,
                            node_to_connect=message.node_to_connect
                            )
                    )
            # Redirects doesn't lock connect requests
            self.lock_connection_request = False

        elif change:
            # Sends a change neighbour request to one neighbour
            yield from self.send(
                    Message("change_neighbour",
                            from_node=self,
                            to_node=to_node,
                            neighbour=message.node_to_connect
                            )
                    )

            # And change the other neighbour to the new node
            if right:
                self.right_node = message.node_to_connect
            else:
                self.left_node = message.node_to_connect

            yield from self.send(
                    Message("new_neighbour",
                            from_node=self,
                            to_node=message.node_to_connect,
                            neighbour=self
                            )
                    )

    def manage_change_neighbour_request(self, message):
        new_neighbour_id = message.neighbour.get_id()

        if new_neighbour_id > self.get_id():
            self.right_node = message.neighbour
        elif new_neighbour_id < self.get_id():
            self.left_node = message.neighbour

        print(self.display_neighbours())

        yield from self.send(
                    Message("new_neighbour",
                            from_node=self,
                            to_node=message.neighbour,
                            neighbour=self
                            )
                    )

    def manage_new_neighbour(self, message):
        if message.neighbour.get_id() > self.get_id():
            self.right_node = message.neighbour
        elif message.neighbour.get_id() < self.get_id():
            self.left_node = message.neighbour

        if self.left_node is not None and self.right_node is not None:
            self.connected = True
            self.waiting_for_connection = False
            self.lock_connection_request = False

            # Send a success message to our neighbours
            yield from self.send_success_insert()

        print(self.display_neighbours())
        yield self.env.timeout(1)

    def send_success_insert(self):
        """Send a success message to our neighbours after joining the DHT"""
        message = Message(
                type="success_join", to_node=self.left_node, from_node=self
                )
        yield from self.send(message)
        message.to_node = self.right_node
        yield from self.send(message)

    def run(self, random_node: 'Node' = None):
        """
        Run the current node
        Parameter:
            - random_node: a random node to connect to.
        """
        while True:
            if self.connected or self.waiting_for_connection:
                # Read received messages
                yield from self.process_messages()
            else:
                yield from self.send(
                        Message("connect",
                                from_node=self,
                                to_node=random_node,
                                node_to_connect=self
                                )
                        )

    def display_neighbours(self):
        """Display neighbours of the nodes"""
        return f"{self.left_node} <- {self} -> {self.right_node}"

    def __repr__(self):
        """Display the node with its color"""
        return f"{self.color}{self.get_id()}{bright_white}"


class DHT(object):
    def __init__(self, env):
        # DHT attributes
        self.nodes: list[Node] = []
        self.processes = []

        node1 = Node(env, 1, color=blue, connected=True)
        node49 = Node(env, 49, color=red, connected=True)
        node50 = Node(env, 50, color=cyan, connected=True)

        node1.left_node = node50
        node1.right_node = node49
        node49.left_node = node1
        node49.right_node = node50
        node50.left_node = node49
        node50.right_node = node1

        node2 = Node(env, 2, color=green)
        node3 = Node(env, 3, color=magenta)

        self.nodes.append(node1)
        self.nodes.append(node2)
        self.nodes.append(node3)
        self.nodes.append(node49)
        self.nodes.append(node50)

        # Simpy config
        self.env = env
        self.register_processes()

    def get_connected_nodes(self) -> list[Node]:
        """Return the list of connected nodes"""
        res = []
        for node in self.nodes:
            if node.connected:
                res.append(node)

        return res

    def register_processes(self):
        """Register every processes into simpy environment"""
        for node in self.nodes:
            random_node = None

            if not node.connected:
                random_node = random.choice(self.get_connected_nodes())

            self.processes.append(
                    env.process(node.run(random_node=random_node))
                    )


env = simpy.Environment()
dht = DHT(env)

env.run(until=200)

print("final state of the DHT")

G = nx.Graph()

for node in dht.nodes:
    print(node.display_neighbours())
    G.add_edge(node.left_node.get_id(), node.get_id())
    G.add_edge(node.get_id(), node.right_node.get_id())

nx.draw(G, with_labels=True)
# plt.show()
