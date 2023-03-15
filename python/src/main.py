import simpy
import random

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
                 node_to_connect: 'Node' = None,
                 neighbour: 'Node' = None):
        # Message types:
        # 'connect': connection request
        # 'change_neighbour': change of neighbour request
        # 'new_neighbour': new neighbour request

        self.type: str = type
        self.from_node: 'Node' = from_node
        self.node_to_connect = node_to_connect

        self.neighbour = neighbour

    def __repr__(self):
        return f"{self.type}"


class Node(object):
    def __init__(self, env, i, color, connected=False):
        self.env = env

        # Implementation
        self.left_node: Node = None
        self.right_node: Node = None
        self.id = i
        self.color = color
        self.message_queue: list[Message] = []

        # Status variables
        self.connected = connected
        self.waiting_for_connection = False
        self.lock_connection_request = False

    def get_id(self):
        return self.id

    def send(self, message: Message, to_node: 'Node'):
        print(
            f"[{self.env.now}][{message.type}] \
            {message.from_node} -> {to_node}"
        )
        yield self.env.timeout(random.randint(1, 10))
        to_node.receive(message)

    def receive(self, message: Message):
        print(
            f"[{self.env.now}][{message.type}] \
            {self} <- {message.from_node}"
        )
        self.message_queue.append(message)

    def process_messages(self):
        for message in self.message_queue:
            if "connect" in message.type:
                yield from self.manage_connection_request(message)
            elif "change_neighbour" in message.type:
                yield from self.manage_change_neighbour_request(message)
            elif "new_neighbour" in message.type:
                yield from self.manage_new_neighbour(message)

        self.message_queue.clear()

    def manage_connection_request(self, message):
        self.lock_connection_request = True

        node_id = message.from_node.get_id()

        redirect = False
        change = False
        right = None
        to_node = None

        # Checking redirection
        if node_id > self.right_node.get_id():
            redirect = True
            to_node = self.right_node
        elif node_id < self.left_node.get_id():
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
                            node_to_connect=message.node_to_connect
                            ),
                    to_node=to_node
                    )
        elif change:
            yield from self.send(
                    Message("change_neighbour",
                            self,
                            neighbour=message.node_to_connect),
                    to_node=to_node
                    )

            if right:
                self.right_node = message.node_to_connect
            else:
                self.left_node = message.node_to_connect

            yield from self.send(
                    Message("new_neighbour",
                            from_node=self,
                            neighbour=self),
                    to_node=message.node_to_connect
                    )

    def manage_change_neighbour_request(self, message):
        new_neighbour_id = message.neighbour.get_id()

        if new_neighbour_id > self.get_id():
            self.right_node = message.from_node
        elif new_neighbour_id < self.get_id():
            self.left_node = message.from_node

        yield from self.send(
                    Message("new_neighbour",
                            from_node=self,
                            neighbour=self),
                    to_node=message.neighbour
                    )

    def manage_new_neighbour(self, message):
        if message.neighbour.get_id() > self.get_id():
            self.right_node = message.neighbour
        elif message.neighbour.get_id() < self.get_id():
            self.left_node = message.neighbour

        print(self.display_neighbours())
        yield self.env.timeout(1)

    def run(self, random_node: 'Node' = None):
        while True:
            if self.connected or self.waiting_for_connection:
                # Lire les message recus
                if len(self.message_queue) != 0:
                    yield from self.process_messages()

                yield self.env.timeout(random.randint(1, 10))
            else:
                self.waiting_for_connection = True

                yield from self.send(
                        Message("connect",
                                from_node=self,
                                node_to_connect=self
                                ),
                        to_node=random_node
                        )

    def display_neighbours(self):
        return f"{self.left_node} <- {self} -> {self.right_node}"

    # Displayinc the node
    def __repr__(self):
        return f"{self.color}{self.get_id()}{bright_white}"


class DHT(object):
    def __init__(self, env):
        # DHT attributes
        self.nodes: list[Node] = []
        self.processes = []

        node1 = Node(env, 1, color=blue, connected=True)
        node4 = Node(env, 4, color=red, connected=True)

        node1.left_node = node4
        node1.right_node = node4
        node4.left_node = node1
        node4.right_node = node1

        node2 = Node(env, 2, color=green)
        node3 = Node(env, 3, color=magenta)

        self.nodes.append(node1)
        self.nodes.append(node2)
        #self.nodes.append(node3)
        self.nodes.append(node4)

        # Simpy config
        self.env = env
        self.run()

    def get_connected_nodes(self) -> list[Node]:
        res = []
        for node in self.nodes:
            if node.connected:
                res.append(node)

        return res

    def run(self):
        for node in self.nodes:
            random_node = None

            if not node.connected:
                random_node = random.choice(self.get_connected_nodes())

            self.processes.append(
                    env.process(node.run(random_node=random_node))
                    )


env = simpy.Environment()
dht = DHT(env)

env.run()
