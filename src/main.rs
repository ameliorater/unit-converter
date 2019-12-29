use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use std::{fs, io};
use rand::Rng;
use petgraph::visit::{IntoNodeReferences};
use petgraph::algo::{has_path_connecting, astar};
use petgraph::{graph};
use std::cmp::min;
use petgraph::dot::{Config, Dot};
#[windows_subsystem = "console"]
#[allow(mutable_borrow_reservation_conflict)]

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct Unit {
    full_name: String,
    ab_name: String
}
impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.ab_name.len() < 6 {
            //doesn't print random id abbreviation placeholders
            write!(f, "{}({})", self.full_name, self.ab_name)
        } else {
            write!(f, "{}", self.full_name)
        }
    }
}

fn main() {
    //a file with this name and the correct formatting must be present in the working directory
    //  file format:
    //  - case insensitive
    //  - abbreviations optional in general, and also after first time unit is input
    //  - spaces in unit names okay
    //  - newlines to space out sections okay

    let input_table = fs::read_to_string("table.txt")
        .expect("Something went wrong reading the file");

    let graph = create_graph(input_table);

    //print graph info (for debugging)
//    println!("Graph node count: {}", graph.node_count());
//    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeIndexLabel]));
//    for i in 0..graph.edge_count() {
//        println!("Edge {} weight: {:?}", i, graph.edge_weight(EdgeIndex::new(i)));
//    }
//    //draw_graph(&graph);

    //initial prompt
    let mut prompt_string = String::from("Please enter a unit conversion: \
    \n(example: '2.4 meters in mm' or '42 furlongs/fortnight to mi/hour')\n\
    (you can also enter 'see units' to see all available units and abbreviations)");

    //loop indefinitely (user must terminate program by closing/stopping)
    'outer: loop {
        //print prompt and collect user input
        let mut input = String::new();
        println!("{}", prompt_string);
        io::stdin().read_line(&mut input).expect("Not a string");
        let mut input: String = String::from(input.trim()); //trim whitespace

        //check if input is requesting unit list (allow typos)
        if get_edit_distance(&input, &String::from("see units")) < 5 {
            print_units(&graph);
            continue
        }

        //separate numerical value
        input = input.replacen(" ", "\n", 1);

        //complex unit conversion (ex: km/hr)
        let mut complex_conversion: bool = false;
        let complex_words = vec!["/", " per "];
        for word in complex_words {
            if input.contains(word) {
                input = input.replace(word, "\n");
                complex_conversion = true;
            }
        }

        //split input into lines for each unit name
        let conversion_words: Vec<&str> = vec![" to ", " in ", " -> "];
        for word in conversion_words {
            input = input.replace(word, "\n");
        }

        //trim any leading or trailing whitespace
        input = input.trim().parse().unwrap();

        //parse input value and unit names
        let mut elements = input.lines();
        let input_val: f64 = elements.next().unwrap().parse().unwrap();
        let mut names: Vec<String> = Vec::new(); //will contain from_name, from_name_2, to_name, to_name_2
        names.push(elements.next().unwrap().to_lowercase());
        if complex_conversion { names.push(elements.next().unwrap().to_lowercase()); }
        names.push(elements.next().unwrap().to_lowercase());
        if complex_conversion { names.push(elements.next().unwrap().to_lowercase()); }

        //find graph nodes corresponding with unit names, if they exist
        let mut indices: Vec<NodeIndex> = Vec::new(); //will contain from_index, from_index_2, to_index, to_index_2
        for name in &names {
            let name_str = &String::from(name.trim());
            if let Some(index) = get_node_from_name(&graph, &name_str, 2) {
                indices.push(index);
            } else {
                println!("{} is not a valid unit\n", name_str);
                continue 'outer
            }
        }

        //used for ranging through units and finding conversion factors
        //different if it's doing a complex conversion (ex: km/hr)
        let mut unit_pairs: Vec<(usize, usize)> = vec![(0, 1)];
        if complex_conversion {
            unit_pairs = vec![(0, 2), (1, 3)]
        }

        //find all conversion factors for this path of conversions and add to vec
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

        //use conversion factors to determine answer
        let mut answer = input_val / *conversion_factors.get(0).unwrap();
        if complex_conversion {
            answer = answer * *conversion_factors.get(1).unwrap();
        }
        print_answer(answer, &names, complex_conversion);

        //change the prompt to something shorter for the next time around
        prompt_string = String::from("Please enter a unit conversion:");
    }
}

//returns the conversion factor to get from one unit to another (none if no possible path)
fn get_conversion_factor (graph: &DiGraph<Unit, f64>, from_index: NodeIndex, to_index: NodeIndex) -> Option<f64> {
    if has_path_connecting(&graph, from_index, to_index, None) {
        //check to see if the conversion is in the table directly
        if graph.contains_edge(from_index, to_index) {
            let edge_index = graph.find_edge(from_index, to_index).unwrap();
            let conversion_factor = *graph.edge_weight(edge_index).unwrap();
            return Some(conversion_factor)
        }

        //a path exists, so find it and find the total conversion factor achieved by traversing it
        let path: Vec<NodeIndex> = astar(&graph, from_index, |n| n == to_index, empty_cost, empty_heuristic).unwrap().1;
        let mut conversion_factor = 1.0;
        let mut previous_node = None;
        for current_node in path {
            if let Some(previous_node) = previous_node {
                let edge_index = graph.find_edge(previous_node, current_node).unwrap();
                conversion_factor *= *graph.edge_weight(edge_index).unwrap();
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
    //graph must be directed to know whether to multiply or divide by a conversion factor
    let mut graph: DiGraph<Unit, f64> = DiGraph::new();
    let lines = input_table.lines();

    for line in lines { //should loop twice
        //allows blank lines in input file
        if line.trim() == "" {
            continue
        }

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
            //random abbreviation id created if no abbreviation provided (to avoid false matches)
            let random_ab_id = rand::thread_rng().gen_range(0, 100_000_000);
            let mut ab_name = String::from(random_ab_id.to_string());
            if unit.contains("(") { //if abbreviation was listed
                ab_name = unit_parts.next().unwrap()
                .replace("(", "").replace(")", "").to_lowercase();
                //search by abbreviation (faster, guaranteed to be unique)
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

        //divide input values to get conversion factor between units
        let conversion_factor = vals.get(0).unwrap()/vals.get(1).unwrap();
        //add an edge to the graph connecting the two nodes joined by this entry
        graph.add_edge(*nodes.get(0).unwrap(), *nodes.get(1).unwrap(), conversion_factor);
    }

    //before returning the graph, populate it with reversed edges
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

//checks for matches of both full name and abbreviation (accounts for plural suffixes/typos with edit distance)
fn get_node_from_name(graph: &DiGraph<Unit, f64>, unit_name: &String, allowed_distance: u64) -> Option<NodeIndex> {
    let mut matching_nodes: Vec<NodeIndex> = Vec::new(); //nodes that match name perfectly
    let mut edit_distances: Vec<(u64, u64)> = Vec::new(); //full and ab name differences from input
    for node in graph.node_references() {
        if &node.1.full_name == unit_name || &node.1.ab_name == unit_name {
            //if exact match, add to list
            matching_nodes.push(node.0);
        }
        //store edit distances for future reference (used if no exact matches are found)
        edit_distances.push((get_edit_distance(&unit_name.clone(), &node.1.full_name.clone()),
                               get_edit_distance(&unit_name.clone(), &node.1.ab_name.clone())))
    }

    if matching_nodes.len() == 1 {
        return Some(*matching_nodes.get(0).unwrap())
    } else if matching_nodes.len() == 0 && allowed_distance > 0 && unit_name.len() > 3 {
        //if there were no matches, we are allowed to check char differences, and it the name isn't
        //  already too short to reasonably cut off characters (not an abbreviation)...
        let mut min_dist: u64 = 1000; //minimum edit distance found between input and all other names
        let mut index_min_dist = 0; //index of node that is most similar to input
        for i in 0..edit_distances.len() {
            //check edit distances of full names and acronyms
            let dist = min(edit_distances.get(i).unwrap().0, edit_distances.get(i).unwrap().1);
            if dist < min_dist {
                min_dist = dist;
                index_min_dist = i;
            }
        }
        if min_dist > allowed_distance {
            //too different, not confident that unit exists
            return None
        } else {
            //return index of node with a name or abbreviation most similar to the input
            let node = graph.node_references().nth(index_min_dist).unwrap().0;
            return Some(node)
        }
    } else {
        //return None if there are multiple matches
        //or, if there were no matches, but edit distance checking is not allowed
        return None
    }
}

//NOTE: this method heavily based on: https://www.geeksforgeeks.org/edit-distance-dp-5/
//uses Levenshtein distance to calculate the minimum number of insertions, deletions,
//  or substitutions needed to convert one string into another
fn get_edit_distance (string1: &String, string2: &String) -> u64 {
    let m: usize = string1.len();
    let n: usize = string2.len();
    let mut dp: Vec<Vec<u64>> = vec![vec![0; n+1]; m+1];

    for i in 0..=m {
        for j in 0..=n {
            if i == 0 {
                dp[i][j] = j as u64;
            } else if j == 0 {
                dp[i][j] = i as u64;
            } else if string1.chars().nth(i-1) == string2.chars().nth(j-1) {
                dp[i][j] = dp[i-1][j-1];
            } else {
                dp[i][j] = 1 + min( min(dp[i][j - 1], dp[i - 1][j]),dp[i - 1][j - 1]);
            }
        }
    }
    dp[m][n]
}

//formats and prints final answer and units
fn print_answer (mut answer: f64, names: &Vec<String>, complex_conversion: bool) {
    if complex_conversion {
        if answer > 0.00001 {
            answer = (answer * 100000_f64).round() / 100000_f64; //round to three decimal places
        }
        println!("{} {}/{}\n", answer, names.get(2).unwrap().trim(), names.get(3).unwrap().trim()); //round to three decimal places
    } else {
        if answer > 0.00001 {
            answer = (answer * 100000_f64).round() / 100000_f64; //round to three decimal places
        }
        println!("{} {}\n", answer, names.get(1).unwrap().trim());
    }
}

//prints list of all available units and their abbreviations, sorted by input order in table
fn print_units(graph: &DiGraph<Unit, f64>) {
    println!("\n\n");
    for node in graph.node_references() {
        println!("{}", node.1);
    }
    println!("\n\n");
}

//to allow use of astar algorithm
fn empty_heuristic<N>(_nid: N) -> u64 { 0 }
fn empty_cost(_er : graph::EdgeReference<f64>) -> u64 { 0 }