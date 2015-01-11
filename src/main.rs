extern crate "rustc-serialize" as rustc_serialize;

use std::os;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use rustc_serialize::{json, Encodable};
use std::io::File;

#[derive(Copy, Eq, PartialEq)]
struct State {
    cost: u64,
    position: u64,
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn shortest_path(graph: &HashMap<u64, Vec<&Edge>>, start: u64, goal: u64) -> Option<Vec<u64>> {
    let mut costs = HashMap::new();
    let mut previous_nodes = HashMap::new();
    let mut heap = BinaryHeap::new();

    costs.insert(start, 0);
    heap.push(State { cost: 0, position: start });

    while let Some(State { cost, position }) = heap.pop() {
        if position == goal { break; }

        let cost_is_larger_than_current_best = match costs.get(&position) {
            Some(current_cost) if cost > *current_cost => true,
            _ => false
        };
     
        if cost_is_larger_than_current_best { continue; }

        let edges = graph.get(&position);
        if edges.is_none() { continue; }

        for edge in edges.unwrap().iter() {
            let next = State { cost: cost + edge.weight, position: edge.to };

            let cost_is_smaller_than_current_best = match costs.get(&next.position) {
                Some(current_cost) if next.cost < *current_cost => true,
                None => true,
                _ => false
            };
     
            if cost_is_smaller_than_current_best { 
                heap.push(next);
                costs.insert(next.position, next.cost);
                previous_nodes.insert(next.position, position);
            }
        }
    }

    if previous_nodes.get(&goal).is_none() { return None }

    let mut reverse_path = vec![goal];
    let mut current_path_node = goal;
    while let Some(node) = previous_nodes.get(&current_path_node) {
        reverse_path.push(*node);
        current_path_node = *node;
    }

    let mut path = vec![];
    while let Some(node) = reverse_path.pop() {
        path.push(node);
    }

    return Some(path);
}

#[derive(RustcDecodable)]
struct Edge {
    from: u64,
    to: u64,
    weight: u64
}

#[derive(RustcEncodable, RustcDecodable)]
struct Journey {
    from: u64,
    to: u64,
    route: Option<Vec<u64>>
}

fn read_as_string(file_name: &str) -> Box<String> {
    let path = Path::new(file_name);
    let mut file = match File::open(&path) {
        Err(why) => panic!("Could not open {}: {}", file_name, why.desc),
        Ok(file) => file,
    };
    return match file.read_to_string() {
        Err(why) => panic!("Could not read {}: {}", file_name, why.desc),
        Ok(string) => Box::new(string),
    };
}

fn main() {

    let args = os::args();
    if args.len() != 3 {
        print!("Usage: {} [graph] [journeys] \n", args[0]);
        return;
    }

    let graph_json = read_as_string(args[1].as_slice());
    let journeys_json = read_as_string(args[2].as_slice());

    let edges: Vec<Edge> = match json::decode((*graph_json).as_slice()) {
        Err(_) => panic!("Could not parse graph!"),
        Ok(edges) => edges,
    };

    let mut graph: HashMap<u64, Vec<&Edge>> = HashMap::new();
    for edge in edges.iter() {
        if graph.contains_key(&edge.from) {
            graph.get_mut(&edge.from).unwrap().push(edge);
        } else {
            graph.insert(edge.from, vec![edge]);
        }
    }

    let journeys: Vec<Journey> = match json::decode((*journeys_json).as_slice()) {
        Err(_) => panic!("Could not parse journeys!"),
        Ok(journeys) => journeys,
    };

    let journeys_with_routes = journeys.iter()
        .map(|journey| {
            let path = shortest_path(&graph, journey.from, journey.to);
            return Journey { from: journey.from, to: journey.to, route: path };
        })
        .collect::<Vec<Journey>>();

    print!("{}\n", json::encode(&journeys_with_routes));
}

