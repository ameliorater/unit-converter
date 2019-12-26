use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use std::{fs, io};
use petgraph::visit::{IntoNodeReferences, Bfs};
use petgraph::data::Element::Node;
use petgraph::dot::{Dot, Config};
use petgraph::algo::{has_path_connecting, all_simple_paths, astar};
use petgraph::{Graph, Undirected, graph};
use std::borrow::BorrowMut;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct Unit {
    full_name: String,
    ab_name: String
}

fn main() {
    let input_table = fs::read_to_string("table.txt")
        .expect("Something went wrong reading the file");

    let graph = create_graph(input_table);

    //debugging print statements
//    println!("Input table: \n{}", lines.next().unwrap());
//    println!("Graph node count: {}", graph.node_count());
//    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));
//    for i in 0..graph.edge_count() {
//        println!("Edge {} weight: {:?}", i, graph.edge_weight(EdgeIndex::new(i)));
//    }

    loop {
        let mut input = String::new();
        println!("Please enter a unit conversion: \n(example: 2.4 meters in mm)");
        io::stdin().read_line(&mut input).expect("Not a string");
        let input = input.trim(); //trim whitespace
        let mut elements = input.split_whitespace();
        let input_val: f32 = elements.next().unwrap().parse().unwrap();
        let mut first_name = elements.next().unwrap().to_lowercase();
        if first_name.ends_with("s") && first_name != String::from("s") {
            first_name.remove(first_name.len() - 1); //remove "s" from end of name
        }
        elements.next(); //clear intermediary word (in, to, etc.)
        let mut second_name = elements.next().unwrap().to_lowercase();
        if second_name.ends_with("s") && second_name != String::from("s") {
            second_name.remove(second_name.len() - 1); //remove "s" from end of name
        }


        let from_index = get_node_index(&graph, &first_name).unwrap();
        let to_index = get_node_index(&graph, &second_name).unwrap();
        if has_path_connecting(&graph, from_index, to_index, None) {
            //check to see if the conversion is in the table
            if graph.contains_edge(from_index, to_index) {
                let edge_index = graph.find_edge(from_index, to_index).unwrap();
                println!("{} {}", input_val / *graph.edge_weight(edge_index).unwrap(), second_name);
                continue;
            }

            let path:  Vec<NodeIndex> = astar(&graph, from_index, |n| n == to_index, empty_cost, empty_heuristic).unwrap().1;
            let mut conversion_factor = 1.0;
            let mut previous_node = None;
            for current_node in path {
                if let Some(previous_node) = previous_node {
                    let edge_index = graph.find_edge(previous_node, current_node).unwrap();
                    conversion_factor /= *graph.edge_weight(edge_index).unwrap();
                    //println!("Previous node: {:?}  Current node: {:?} Dividing by: {}", previous_node, current_node, *graph.edge_weight(edge_index).unwrap());
                }
                if current_node == to_index {
                    //target node reached
                    break
                }
                previous_node = Some(current_node);
            }
            println!("{} {}", input_val * conversion_factor, second_name);
        } else {
            println!("Not a valid conversion");
        }
    }
}

//generates a unit conversion graph given an input table in specific format
fn create_graph (input_table: String) -> DiGraph<Unit, f32> {
    let mut graph: DiGraph<Unit, f32> = DiGraph::new();
    let mut lines = input_table.lines();

    for line in lines {
        let mut line_elements = line.split_whitespace();
        let first_val: f32 = line_elements.next().unwrap().parse().unwrap();
        let mut full_name = line_elements.next().unwrap().to_lowercase();
        if full_name.ends_with("s") && full_name != String::from("s") { //make sure it isn't s for seconds
            full_name.remove(full_name.len() - 1); //remove "s" from end of name
        }
        let ab_name = line_elements.next().unwrap()
            .replace("(", "").replace(")", "").to_lowercase();
        let mut node_1 = NodeIndex::new(0);
        if let Some(node_index) = get_node_index(&graph, &full_name) {
            node_1 = node_index;
        } else {
            node_1 = graph.add_node(Unit { full_name, ab_name });
        }

        line_elements.next(); //clear equals sign

        let second_val: f32 = line_elements.next().unwrap().parse().unwrap();
        let mut full_name = line_elements.next().unwrap().to_lowercase();
        if full_name.ends_with("s")  && full_name != String::from("s") {
            full_name.remove(full_name.len() - 1); //remove "s" from end of name
        }
        let ab_name = line_elements.next().unwrap()
            .replace("(", "").replace(")", "").to_lowercase();
        let mut node_2= NodeIndex::new(0);
        if let Some(node_index) = get_node_index(&graph, &full_name) {
            node_2 = node_index;
        } else {
            node_2 = graph.add_node(Unit { full_name, ab_name });
        }

        let conversion_factor = first_val/second_val;
        graph.add_edge(node_1, node_2, conversion_factor);
    }
    add_reverse_edges(graph)
}

//adds reverse conversions to the graph
fn add_reverse_edges (mut graph: DiGraph<Unit, f32>) -> DiGraph<Unit, f32> {
    for edge_index in graph.edge_indices() {
        let edge_endpoints = graph.edge_endpoints(edge_index).unwrap();
        let from_index = edge_endpoints.0;
        let to_index = edge_endpoints.1;
        let original_weight = graph.edge_weight(edge_index).unwrap();
        if !graph.contains_edge(to_index, from_index) {
            graph.add_edge(to_index, from_index, 1.0/ *original_weight);
        }
    }
    graph
}

//checks for matches of both full name and abbreviation
fn get_node_index (graph: &DiGraph<Unit, f32>, unit_name: &String) -> Option<NodeIndex> {
    for node in graph.node_references() {
        if &node.1.full_name == unit_name || &node.1.ab_name == unit_name {
            let node_index = node.0;
            return Some(node_index)
        }
    }
    return None
}

//to allow use of astar algorithm
fn empty_heuristic<N>(nid: N) -> u32 { 0 }
fn empty_cost(er : graph::EdgeReference<f32>) -> u32 { 0 }