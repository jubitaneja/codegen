// Merged prefix tree

use std::collections::HashMap;
use patternmatcher::{self, Arena, Node, NodeType, NodeID, Node_Index};


#[derive(Clone)]
pub struct MergedArena {
    pub merged_tree: Vec<Node>,
    pub hmap: HashMap<String, usize>,
}

impl MergedArena {
    pub fn build_root_node(&mut self) -> Node {
        Node {
            node_type: NodeType::match_root,
            node_value: "root".to_string(),
            id: 0,
            next: Some(Vec::new()),
        }
    }

    pub fn get_first_node_of_opt_pattern(&mut self, opt: Vec<Node>) -> Node {
        if opt.len() != 0 {
            opt[0].clone()
        } else {
            panic!("Error: optimization pattern has no nodes")
        }
    }

    pub fn get_value_of_node(&mut self, node: Node) -> String {
        node.node_value
    }

    pub fn get_id_of_node(&mut self, node: Node) -> usize {
        node.id
    }

    pub fn update_hash_map(&mut self, value: String, id: usize) {
        self.hmap.insert(value, id);
    }

    pub fn update_next_nodes_list(&mut self, mut node: Node, next_id: usize) -> Node {
        if let Some(mut node_next_list) = node.next {
            node_next_list.push(NodeID{ index: next_id });
            node.next = Some(node_next_list);
            node
        } else {
            panic!("Error: Node's next list is None, should be atleast an empty vec<NodeID>");
        }
    }

    // add a new node to arena
    pub fn add_node_to_arena(&mut self, node: Node) {
        self.merged_tree.push(node);
    }

    // when the node exists in arena already, update it
    pub fn update_node_in_arena(&mut self, updated_node: Node) {
        for n in 0 .. self.merged_tree.len() {
            if self.merged_tree[n].id == updated_node.id {
                self.merged_tree[n].next = updated_node.next;
                break;
            }
        }
    }

    pub fn find_node_with_id_in_arena(&mut self, node_id: usize) -> Node {
        // FIXME: not a good implementation to traverse and exit if id is found
        let mut found_node = Node {
                                   node_type: NodeType::match_none,
                                   node_value: "dummy".to_string(),
                                   id: 10000,
                                   next: None,
                                  };
        for n in 0 .. self.merged_tree.len() {
            if self.merged_tree[n].id == node_id {
                found_node = self.merged_tree[n].clone();
                break;
            } else {
                panic!("Error: node with id: {} doesn't exist in merged_arena", node_id);
            }
        }
        found_node
    }

    pub fn node_has_any_connection(&mut self, node: Node) -> bool {
        let mut connections = false;
        if let Some(nodes_list) = node.next {
            if nodes_list.len() == 0 {
                connections = false;
            } else {
                connections = true;
            }
        }
        connections
    }

    pub fn is_node_value_already_in_hash_map(&mut self, val: String) -> bool {
        if self.hmap.contains_key(&val) {
            true
        } else {
            false
        }
    }
}

pub fn generate_merged_prefix_tree(single_tree: Vec<Node>, mut merged_arena: MergedArena) -> MergedArena {
    if merged_arena.merged_tree.len() == 0 {
        let root_node = merged_arena.build_root_node();
        merged_arena.add_node_to_arena(root_node);
    }

    let first_node = merged_arena.get_first_node_of_opt_pattern(single_tree.clone());
    let current_val = merged_arena.get_value_of_node(first_node.clone());
    let current_id = merged_arena.get_id_of_node(first_node.clone());

    let found_root = merged_arena.find_node_with_id_in_arena(0);
    println!("**** found root node with value = {}", found_root.node_value);
    if let Some(nodes_list) = found_root.next.clone() {
        for x in  0 .. nodes_list.len() {
            println!("root->next ====== {}", nodes_list[x].index);
        }
    }

    if !merged_arena.node_has_any_connection(found_root.clone()) {
        // case 1
        println!("No connection of root node ------------");
        merged_arena.update_hash_map(current_val, current_id);
        let updated_root = merged_arena.update_next_nodes_list(found_root.clone(), current_id);
        merged_arena.update_node_in_arena(updated_root);
        for n in 0 .. single_tree.len() {
            merged_arena.add_node_to_arena(single_tree[n].clone());
        }
    } else {
        println!("found connection of root node ------------");
        println!("current val for opt2 = {}", current_val);
        if merged_arena.is_node_value_already_in_hash_map(current_val.clone()) {
            // case 2
            println!("Hash key found");
            // already in hashmap
            // find corresponding id of that hash key value = id2
            // traverse in merged tree to find the node with the id: id2
            // now, traverse step by step in single tree and in merged tree to compare node values
            // wherever u find first difference, append the rest of single tree nodes there.
            unimplemented!();
        } else {
            // case 3 - same as case 1
            merged_arena.update_hash_map(current_val, current_id);
            let updated_root = merged_arena.update_next_nodes_list(found_root.clone(), current_id);
            merged_arena.update_node_in_arena(updated_root);
            for n in 0 .. single_tree.len() {
                merged_arena.add_node_to_arena(single_tree[n].clone());
            }
        }
    }

    merged_arena
}
