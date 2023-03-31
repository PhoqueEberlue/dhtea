class Message(object):
    def __init__(self,
                 type: str,
                 from_node,
                 to_node,
                 node_to_connect=None,
                 neighbour=None):

        # Message types:
        # 'connect': connection request
        # 'change_neighbour': change of neighbour request
        # 'new_neighbour': new neighbour request

        self.type: str = type
        self.from_node = from_node
        self.to_node = to_node
        self.node_to_connect = node_to_connect

        self.neighbour = neighbour

    def __repr__(self):
        return f"node_to_connect: {self.node_to_connect} neighbour: {self.neighbour}"
