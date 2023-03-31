from message import Message
from colors import COLORS
import random


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

        # Random timeout before delivering the message
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
                    # Appends connection request to the queue
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

        # Treat one connection request in the queue if the node isn't locked
        # and the queue not empty
        if not self.lock_connection_request and len(self.connection_queue) != 0:
            yield from self.manage_connection_request(
                    self.connection_queue.pop(0)
                    )
        else:
            if len(self.connection_queue) > 0:
                message = self.connection_queue[0]
                print(f"[{self.env.now}][CONNECTION QUEUE] {self} has {message.from_node} in queue")

    def manage_connection_request(self, message):
        """Manage connection request, either redirecting, or inserting as a neighbour"""

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

        # Otherwise, checking which side the node should be added
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
        """
        Manage change neighbour request, also sending a new neighbour
        message to the changed neighbour
        """
        new_neighbour_id = message.neighbour.get_id()

        # Checking which side the neighbour should be replaced
        if new_neighbour_id > self.get_id():
            self.right_node = message.neighbour
        elif new_neighbour_id < self.get_id():
            self.left_node = message.neighbour

        print(self.display_neighbours())

        # Sending the new neighbour that the current node is his
        # new neighbour
        yield from self.send(
                    Message("new_neighbour",
                            from_node=self,
                            to_node=message.neighbour,
                            neighbour=self
                            )
                    )

    def manage_new_neighbour(self, message):
        """
        Manage new neighbour request sending a success message if
        both of the neighbours are not null
        """

        # Checking which side the neighbour should be added
        if message.neighbour.get_id() > self.get_id():
            self.right_node = message.neighbour
        elif message.neighbour.get_id() < self.get_id():
            self.left_node = message.neighbour

        # If both neighbours are connected
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

        # Changing the dest
        message.to_node = self.right_node
        yield from self.send(message)

    def run(self, random_node: 'Node' = None):
        """
        Run the current node. This is the main function of the class
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
        return f"{self.color}{self.get_id()}{COLORS['bright_white']}"
