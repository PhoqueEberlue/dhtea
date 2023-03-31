from dht import DHT
import networkx as nx
import simpy
import matplotlib.pyplot as plt


# Creating our environment
env = simpy.Environment()
dht = DHT(env)

# Simple scenario with 5 nodes with nice colors
dht.create_simple_scenario()

# Creates a more advanced scenario with n nodes
# dht.create_advanced_scenario(15)

env.run(until=200)

print("final state of the DHT")

G = nx.Graph()

for node in dht.nodes:
    print(node.display_neighbours())
    G.add_edge(node.left_node.get_id(), node.get_id())
    G.add_edge(node.get_id(), node.right_node.get_id())


# Uncomment to show the graph
# nx.draw(G, with_labels=True)
# plt.show()
