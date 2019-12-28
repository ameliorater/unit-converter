use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use std::{fs, io};
use rand::Rng;
use petgraph::visit::{IntoNodeReferences};
use petgraph::algo::{has_path_connecting, astar};
use petgraph::{graph};
use std::cmp::min;
use petgraph::dot::{Config, Dot};
#[allow(mutable_borrow_reservation_conflict)]

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct Unit {
    full_name: String,
    ab_name: String
}
impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(Name: {}, Abbreviation: {})", self.full_name, self.ab_name)
    }
}

fn main() {
    let input_table = fs::read_to_string("table.txt")
        .expect("Something went wrong reading the file");

    let graph = create_graph(input_table);

    //debugging print statements
//    println!("Graph node count: {}", graph.node_count());
//    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));
//    for i in 0..graph.edge_count() {
//        println!("Edge {} weight: {:?}", i, graph.edge_weight(EdgeIndex::new(i)));
//    }
//    //draw_graph(&graph);

    let mut prompt_string = String::from("Please enter a unit conversion: \n(example: 2.4 meters in mm)");

    'outer: loop {
        let mut input = String::new();
        println!("{}", prompt_string);
        io::stdin().read_line(&mut input).expect("Not a string");
        let mut input: String = String::from(input.trim()); //trim whitespace

        //complex unit conversion
        let mut complex_conversion: bool = false;
        if input.contains("/") || input.contains("per") {
            input = input.replace("/", " ");
            input = input.replace("per", " ");
            complex_conversion = true;
        }

        let mut elements = input.split_whitespace();
        let input_val: f64 = elements.next().unwrap().parse().unwrap();
        let mut names: Vec<String> = Vec::new(); //will contain from_name, from_name_2, to_name, to_name_2
        names.push(elements.next().unwrap().to_lowercase());
        if complex_conversion { names.push(elements.next().unwrap().to_lowercase()); }
        elements.next(); //clear intermediary word (in, to, etc.)
        names.push(elements.next().unwrap().to_lowercase());
        if complex_conversion { names.push(elements.next().unwrap().to_lowercase()); }

        let mut indices: Vec<NodeIndex> = Vec::new(); //will contain from_index, from_index_2, to_index, to_index_2
        for name in &names {
            if let Some(index) = get_node_from_name(&graph, &name, 3) {
                //println!("index: {:?}", index);
                indices.push(index);
            } else {
                println!("{} is not a valid unit\n", name);
                continue 'outer
            }
        }

        let mut unit_pairs: Vec<(usize, usize)> = vec![(0, 1)];
        if complex_conversion {
            unit_pairs = vec![(0, 2), (1, 3)]
        }

        let mut conversion_factors: Vec<f64> = Vec::new();
        for pair in unit_pairs {
            let from_index = *indices.get(pair.0).unwrap();
            let to_index = *indices.get(pair.1).unwrap();
            if let Some(conversion_factor) = get_conversion_factor(&graph, from_index, to_index) {
                //println!("conversion factor {}", conversion_factor);
                conversion_factors.push(conversion_factor);
            } else {
                println!("Not a valid conversion\n");
                continue 'outer
            }
        }

        let mut answer = input_val / *conversion_factors.get(0).unwrap();
        if complex_conversion {
            answer = answer * *conversion_factors.get(1).unwrap();
        }
        print_answer(answer, &names, complex_conversion);

        prompt_string = String::from("Please enter a unit conversion:");
    }
}

fn get_conversion_factor (graph: &DiGraph<Unit, f64>, from_index: NodeIndex, to_index: NodeIndex) -> Option<f64> {
    if has_path_connecting(&graph, from_index, to_index, None) {
        //check to see if the conversion is in the table
        if graph.contains_edge(from_index, to_index) {
            let edge_index = graph.find_edge(from_index, to_index).unwrap();
            let conversion_factor = *graph.edge_weight(edge_index).unwrap();
            return Some(conversion_factor)
        }

        let path:  Vec<NodeIndex> = astar(&graph, from_index, |n| n == to_index, empty_cost, empty_heuristic).unwrap().1;
        let mut conversion_factor = 1.0;
        let mut previous_node = None;
        for current_node in path {
            if let Some(previous_node) = previous_node {
                let edge_index = graph.find_edge(previous_node, current_node).unwrap();
                conversion_factor *= *graph.edge_weight(edge_index).unwrap();
                //println!("Previous node: {:?}  Current node: {:?} Dividing by: {}", previous_node, current_node, *graph.edge_weight(edge_index).unwrap());
            }
            if current_node == to_index {
                //target node reached
                break
            }
            previous_node = Some(current_node);
        }
        return Some(conversion_factor)
    } else {
        return None
    }
}

//generates a unit conversion graph given an input table in specific format
fn create_graph (input_table: String) -> DiGraph<Unit, f64> {
    let mut graph: DiGraph<Unit, f64> = DiGraph::new();
    let lines = input_table.lines();

    for line in lines { //should loop twice
        //separates units
        let line_string = line.replace("=", "\n\r");
        let units = line_string.lines();

        let mut nodes: Vec<NodeIndex> = Vec::new();
        let mut vals: Vec<f64> = Vec::new();
        for mut unit in units {
            //separate unit string into value, name, and abbreviation
            unit = unit.trim();
            let mut unit_string = String::from(unit);
            unit_string = unit_string.replace("(", "\n");
            unit_string = unit_string.replace(")", "");
            unit_string = unit_string.replacen(" ", "\n", 1);
            let mut unit_parts = unit_string.split("\n");

            let val_string = unit_parts.next().unwrap().trim();
            vals.push(val_string.parse().unwrap());
            let full_name = unit_parts.next().unwrap().to_lowercase();
            let random_ab_id = rand::thread_rng().gen_range(0, 100_000);
            let mut ab_name = String::from(random_ab_id.to_string());
            if unit.contains("(") { //if abbreviation was included
                ab_name = unit_parts.next().unwrap()
                .replace("(", "").replace(")", "").to_lowercase();
                //search by abbreviation (faster, no edit distance)
                if let Some(node_index) = get_node_from_name(&graph, &ab_name, 0) {
                    nodes.push(node_index);
                } else {
                    nodes.push(graph.add_node(Unit { full_name, ab_name }));
                }
            } else {
                //no abbreviation listed, so search by full name with allowed edit distance of 2
                if let Some(node_index) = get_node_from_name(&graph, &full_name, 2) {
                    nodes.push(node_index);
                } else {
                    nodes.push(graph.add_node(Unit { full_name, ab_name }));
                }
            }
        }

        let conversion_factor = vals.get(0).unwrap()/vals.get(1).unwrap();
        graph.add_edge(*nodes.get(0).unwrap(), *nodes.get(1).unwrap(), conversion_factor);
    }

    add_reverse_edges(graph)
}

//adds reverse conversions to the graph
fn add_reverse_edges (mut graph: DiGraph<Unit, f64>) -> DiGraph<Unit, f64> {
    for edge_index in graph.edge_indices() {
        let edge_endpoints = graph.edge_endpoints(edge_index).unwrap();
        let from_index = edge_endpoints.0;
        let to_index = edge_endpoints.1;
        let original_weight = graph.edge_weight(edge_index).unwrap();
        if !graph.contains_edge(to_index, from_index) {
            graph.add_edge(to_index, from_index, 1.0 / *original_weight);
        }
    }
    graph
}

//checks for matches of both full name and abbreviation (accounts for plural suffixes)
fn get_node_from_name(graph: &DiGraph<Unit, f64>, unit_name: &String, allowed_distance: u64) -> Option<NodeIndex> {
    let mut matching_nodes: Vec<NodeIndex> = Vec::new();
    let mut edit_distances: Vec<(u64, u64)> = Vec::new(); //full and ab name differences
    for node in graph.node_references() {
        if &node.1.full_name == unit_name || &node.1.ab_name == unit_name {
            matching_nodes.push(node.0);
        }
        edit_distances.push((get_edit_distance(unit_name.clone(), node.1.full_name.clone()),
                               get_edit_distance(unit_name.clone(), node.1.ab_name.clone())))
    }

    //return None if there are multiple matches
    if matching_nodes.len() == 1 {
        //println!("returning single match for {}", unit_name);
        return Some(*matching_nodes.get(0).unwrap())
    } else if matching_nodes.len() == 0 && allowed_distance > 0 && unit_name.len() > 3 {
        //if there were no matches, we are allowed to check char differences, and it the name isn't
        //  already too short to reasonably cut off characters (not an abbreviation)...
        let mut min_dist: u64 = 1000;
        let mut index_min_dist = 0;
        for i in 0..edit_distances.len() {
            let dist = edit_distances.get(i).unwrap().0; //use distance with full name only
            if dist < min_dist {
                min_dist = dist;
                index_min_dist = i;
            }
        }
        if min_dist > allowed_distance {
            //too different, unit probably doesn't exist
            //println!("too different {}", unit_name);
            return None
        } else {
            //return node index with min dist
            let node = graph.node_references().nth(index_min_dist).unwrap().0;
            //println!("matched {} with node at {:?} with edit distance of {}", unit_name, node, min_dist);
            return Some(node)
        }
    } else {
        //println!("more than one match found for {}", unit_name);
        return None
    }
}

//uses Levenshtein distance to calculate the minimum number of insertions, deletions,
//  or substitutions needed to convert one string into another
//currently recursive, can be implemented using dp and would be much faster for large strings
fn get_edit_distance (mut string1: String, mut string2: String) -> u32 {
    if string1.len() == 0 {
        return string2.len() as u32;
    }
    if string2.len() == 0 {
        return string1.len() as u32;
    }

    //if last characters are equal
    if string1.chars().nth(string1.len()-1) == string2.chars().nth(string2.len()-1) {
        string1.remove(string1.len()-1);
        string2.remove(string2.len()-1);
        return get_edit_distance(string1, string2);
    }

    let mut temp_string_1 = string1.clone();
    temp_string_1.remove(temp_string_1.len()-1);
    let mut temp_string_2 = string2.clone();
    temp_string_2.remove(temp_string_2.len()-1);
    let substitution = get_edit_distance(temp_string_1, temp_string_2);

    let temp_string_1 = string1.clone();
    let mut temp_string_2 = string2.clone();
    temp_string_2.remove(temp_string_2.len()-1);
    let insertion = get_edit_distance(temp_string_1, temp_string_2);

    let mut temp_string_1 = string1.clone();
    temp_string_1.remove(temp_string_1.len()-1);
    let temp_string_2 = string2.clone();
    let deletion = get_edit_distance(temp_string_1, temp_string_2);

    //return 1 + min possible with each of the three operations
    return 1 + min(substitution, min(insertion, deletion));
}

fn print_answer (mut answer: f64, names: &Vec<String>, complex_conversion: bool) {
    if complex_conversion {
        if answer > 0.00001 {
            answer = (answer * 100000_f64).round() / 100000_f64; //round to three decimal places
        }
        println!("{} {}/{}\n", answer, names.get(2).unwrap(), names.get(3).unwrap()); //round to three decimal places
    } else {
        if answer > 0.00001 {
            answer = (answer * 100000_f64).round() / 100000_f64; //round to three decimal places
        }
        println!("{} {}\n", answer, names.get(1).unwrap());
    }
}

//to allow use of astar algorithm
fn empty_heuristic<N>(_nid: N) -> u64 { 0 }
fn empty_cost(_er : graph::EdgeReference<f64>) -> u64 { 0 }