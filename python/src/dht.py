from node import Node
from colors import COLORS
import random


class DHT(object):
    def __init__(self, env):
        # DHT attributes
        self.nodes: list[Node] = []
        self.processes = []

        # Simpy config
        self.env = env

    def create_simple_scenario(self):
        node1 = Node(self.env, 1, color=COLORS['blue'], connected=True)
        node49 = Node(self.env, 49, color=COLORS['red'], connected=True)
        node50 = Node(self.env, 50, color=COLORS['cyan'], connected=True)

        node1.left_node = node50
        node1.right_node = node49
        node49.left_node = node1
        node49.right_node = node50
        node50.left_node = node49
        node50.right_node = node1

        node2 = Node(self.env, 2, color=COLORS['green'])
        node3 = Node(self.env, 3, color=COLORS['magenta'])

        self.nodes.append(node1)
        self.nodes.append(node2)
        self.nodes.append(node3)
        self.nodes.append(node49)
        self.nodes.append(node50)

        self.register_processes()

    def create_advanced_scenario(self, n):
        node1 = Node(self.env, 1, color=COLORS['blue'], connected=True)
        node49 = Node(self.env, n-2, color=COLORS['red'], connected=True)
        node50 = Node(self.env, n-1, color=COLORS['cyan'], connected=True)

        node1.left_node = node50
        node1.right_node = node49
        node49.left_node = node1
        node49.right_node = node50
        node50.left_node = node49
        node50.right_node = node1

        self.nodes.append(node1)
        self.nodes.append(node49)
        self.nodes.append(node50)

        for i in range(2, n-2):
            self.nodes.append(Node(self.env, i, color=COLORS['bright_white']))

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
                    # Call run function (A generator) for each node
                    self.env.process(node.run(random_node=random_node))
                    )
