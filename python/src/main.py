import simpy
from random import randint

class Node(object):
    def __init__(self, env):
        # Simpy env 
        self.env = env
        self.action = env.process(self.run())

        # Implementation
        self.left_node = None
        self.right_node = None
        self.set_random_ip_port()

    def set_random_ip_port(self):
        x = lambda: randint(1, 256)
        self.port = randint(10000, 25000)
        self.ip = f"{x()}.{x()}.{x()}.{x()}"

    # Main function of the node
    def run(self):
        while True:
            print(f"[{self.__repr__()}] {self.env.now}")
            yield self.env.timeout(3)

    # Displayinc the node
    def __repr__(self):
        return f"{self.ip}:{self.port}"


env = simpy.Environment()

nodes = set()

for i in range(0, 10):
    nodes.add(Node(env))

print(nodes)

env.run(until=10)


