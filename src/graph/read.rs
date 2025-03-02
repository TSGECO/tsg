use graphina::core::types::Graph;

pub fn test_read() {
    // Create a new undirected graph
    let mut graph = Graph::new();

    // Add nodes and edges to the graph
    let n0 = graph.add_node(1);
    let n1 = graph.add_node(1);
    let n2 = graph.add_node(2);
    let n3 = graph.add_node(3);
    graph.add_edge(n0, n1, 1.0);
    graph.add_edge(n1, n2, 1.0);
    graph.add_edge(n2, n3, 1.0);

    // Get the neighbors of node 1
    for neighbor in graph.neighbors(n1) {
        println!("Node 1 has neighbor: {}", neighbor.index());
    }
}
