// Matcher

use std::collections::HashMap;
use lhspatternmatcher::{self, Node, NodeType, NodeID};
use cliftinstbuilder::{self, CtonInst, CtonValueDef,
                       CtonInstKind, CtonOpcode,
                       CtonOperand};

#[derive(Clone)]
pub struct Opt {
    current_entity: String,
    func_str: String,
    scope_stack: Vec<ScopeStack>,
    const_stack: Vec<String>,
}

#[derive(Clone)]
pub struct ScopeStack {
    scope_type: ScopeType,
    level: usize,
}

#[derive(Clone)]
pub enum ScopeType {
    scope_match,
    scope_case,
    scope_func,
}

impl Opt {
    pub fn new() -> Opt {
        Opt {
            current_entity: String::from("inst"),
            func_str: String::from(""),
            scope_stack: Vec::new(),
            const_stack: Vec::new(),
        }
    }

    pub fn generate_header(&mut self, mut count: u32) {
        self.func_str.push_str("fn superopt_");
        self.func_str.push_str(&count.to_string());
        self.func_str.push_str("(pos: &mut FuncCursor, inst: Inst)");
    }

    pub fn append(&mut self, input: String) {
        self.func_str.push_str(&input);
    }

    pub fn set_entity(&mut self, entity: String) {
        self.current_entity = entity;
    }

    pub fn does_level_exist_in_stack(&mut self,
                                     find_level: usize) -> usize {
        let mut index = 0;
        for i in 0 .. self.scope_stack.len() {
            if find_level == self.scope_stack[i].level {
                index = i;
                break;
            } else {
                continue;
            }
        }
        index
    }

    pub fn pop_and_exit_scope_from(&mut self, from: usize) {
        for i in from .. self.scope_stack.len() {
            let stack_elem = self.scope_stack.pop();
            match stack_elem {
                Some(elem) => {
                    self.exit_scope(elem.scope_type, elem.level);
                },
                None => {},
            }
        }
    }

    pub fn push_to_const_stack(&mut self, rhs: String) {
        self.const_stack.push(rhs);
    }

    pub fn pop_from_const_stack(&mut self) -> String {
        match self.const_stack.pop() {
            Some(rhs) => {
                rhs.to_string()
            },
            None => {
                "".to_string()
            },
        }
    }

    pub fn enter_scope(&mut self,
                       scope: ScopeType,
                       current_level: usize) {
        // check for the level in current stack
        // if level is new - not found in stack -
        // push it directly.
        // if level already exists in stack,
        // pop the stack until that level and then
        // push the new level.
        // Debug
        //println!("Current stack before entering scope is: -------");
        //for x in 0 .. self.scope_stack.len() {
           //println!("stack levels pushed so far = {}", self.scope_stack[x].level);
        //}
        //println!("find the level number = {} in stack", current_level);
        let index = self.does_level_exist_in_stack(current_level);
        //println!("Found index from stack == {}", index);
        if index != 0 {
            // index exists
            // pop first
            self.pop_and_exit_scope_from(index);
        }
        // push the level
        self.scope_stack.push(ScopeStack {
                              scope_type: scope.clone(),
                              level: current_level
                              });
        // append the string
        match scope {
            ScopeType::scope_match => {
                self.append(String::from(" {\n"));
            },
            ScopeType::scope_func => {
                self.append(String::from(" {\n"));
            },
            ScopeType::scope_case => {
                self.append(String::from(" => {\n"));
            },
            _ => {
                panic!("Error: No such scope type exists");
            },
        }
    }

    pub fn exit_scope(&mut self, scope: ScopeType, level: usize) {
        match scope {
            ScopeType::scope_match => {
                self.append(String::from("\n}"));
            },
            ScopeType::scope_func => {
                self.append(String::from("\n}"));
            },
            ScopeType::scope_case => {
                self.append(String::from("\n},"));
                // For baseline matcher, there will be always
                // one node at one level. So, we will end
                // up with one if case and else case.
                self.append(String::from("\n_ => {},"));
            },
            _ => {
                panic!("Error: No such scope type exists");
            },
        }
    }

    pub fn is_leaf_node(&mut self, node: Node) -> bool {
        match node.next {
            Some(x) => false,
            None => true,
        }
    }

    pub fn init_dummy_node(&mut self) -> Node {
        Node {
              node_type: NodeType::match_none,
              node_value: "dummy".to_string(),
              width: 0,
              id: <usize>::max_value(),
              var_id: None,
              arg_flag: false,
              level: 0,
              next: None,
             }
    }

    pub fn update_node_with_level(&mut self,
                                  mut node: Node,
                                  level: usize) -> Node {
        node.level = level;
        node
    }

    pub fn update_node_level_in_lhs(&mut self,
                                    updated_node: Node,
                                    nodes: &mut Vec<Node>) {
        for n in 0 .. nodes.len() {
            if nodes[n].id == updated_node.id {
                nodes[n].level = updated_node.level;
                break;
            }
        }
    }

    pub fn find_node_with_id(&mut self,
                             id: usize,
                             nodes: &mut Vec<Node>) -> Node {
        let mut found_node = self.init_dummy_node();
        for i in 0 .. nodes.len() {
            if (id == nodes[i].id) {
                found_node = nodes[i].clone();
                break;
            } else {
                continue;
            }
        }
        found_node
    }

    pub fn set_level_of_all_child_nodes(&mut self,
                                        nodes: &mut Vec<Node>,
                                        n: usize,
                                        current: usize) {
        if let Some(next_nodes) = nodes[n].next.clone() {
            if (next_nodes.len() > 1) {
                panic!("Error: there should be only one node in LHS single tree\n");
            }
            for n in 0 .. next_nodes.len() {
                // It will certainly be one next node,
                // as this is for single LHS vec of nodes
                let id = next_nodes[n].index;
                let mut next_node = self.find_node_with_id(id, nodes);
                let updated_node = self.update_node_with_level(
                                   next_node.clone(),
                                   current + 1);
                self.update_node_level_in_lhs(updated_node.clone(), nodes);
            }
        }
    }
    
    pub fn build_root_node(&mut self) -> Node {
        Node {
            node_type: NodeType::match_root,
            node_value: "root".to_string(),
            width: 0,
            id: 0,
            var_id: None,
            arg_flag: false,
            level: 0,
            next: Some(Vec::new()),
        }
    }

    pub fn take_action(&mut self, rhs: Vec<CtonInst>) {
        let mut insert_inst_str = "".to_string();

        if rhs.len() == 1 {
            let each_inst = rhs[0].clone();
            let mut rhs_const : i32 = 0;

            match each_inst.cops {
                Some(ops) => {
                    for op in ops {
                        match op.const_val {
                            Some(c) => {
                                rhs_const = c;
                            },
                            None => {},
                        }
                    }
                },
                None => {},
            }

            let mut replace_inst_str = "".to_owned();
            replace_inst_str = "pos.func.dfg.replace(".to_owned();
            // FIXME: fix the inst name here
            replace_inst_str += &"inst".to_owned();
            replace_inst_str += &").".to_owned();
            replace_inst_str += &"iconst(".to_owned();
            // FIXME: fix the width part
            // Note: Width is not set in CliftInst operands
            // for constant type, only const_val exists. 
            // Only parser has the correct width for constant, see
            // how you pass that information further to CliftInst struct
            replace_inst_str += &"width".to_owned();
            replace_inst_str += &", ".to_owned();
            replace_inst_str += &rhs_const.to_string();
            replace_inst_str += &"); ".to_owned();
            self.func_str.push_str(&replace_inst_str);
        } else {
            for inst in 0 .. rhs.len()-2 {
                let each_inst = rhs[inst].clone();
                insert_inst_str = "let inst".to_owned();
                insert_inst_str += &inst.to_string();
                insert_inst_str += &" = pos.ins().".to_owned();
                insert_inst_str += &cliftinstbuilder::get_clift_opcode_name(
                                   each_inst.opcode);
                insert_inst_str += &"(".to_owned();
                // [Pending] FIXME: fix the args names and count of args here
                insert_inst_str += &"args[0], args[1]".to_owned();
                insert_inst_str += &");\n".to_owned();
                self.func_str.push_str(&insert_inst_str);
            }
            let mut replace_inst_str = "".to_owned();
            for inst in rhs.len()-2 .. rhs.len()-1 {
                let each_inst = rhs[inst].clone();
                replace_inst_str = "pos.func.dfg.replace(".to_owned();
                // FIXME: fix the inst name here
                replace_inst_str += &"inst".to_owned();
                replace_inst_str += &").".to_owned();
                replace_inst_str += &cliftinstbuilder::get_clift_opcode_name(
                                    each_inst.opcode);
                replace_inst_str += &"(".to_owned();
                // FIXME: fix the args names and count of args here
                replace_inst_str += &"args[0], args[1]".to_owned();
                replace_inst_str += &");\n".to_owned();
                self.func_str.push_str(&replace_inst_str);
            }
        }
    }

    pub fn get_argument_counter(&mut self, mut count: u32) -> u32 {
        count = count + 1;
        self.append(String::from(count.to_string()));
        count
    }

    pub fn get_const_counter(&mut self, mut count: u32) -> u32 {
        count = count + 1;
        count
    }

}

pub fn is_node_actionable(node_id: usize,
                          table: HashMap<usize,
                          Vec<CtonInst>>) -> bool {
    if table.contains_key(&node_id) {
        true
    } else {
        false
    }
}

pub enum IntCC {
    Equal,
    NotEqual,
    SignedLessThan,
    SignedGreaterThanOrEqual,
    SignedGreaterThan,
    SignedLessThanOrEqual,
    UnsignedLessThan,
    UnsignedGreaterThanOrEqual,
    UnsignedGreaterThan,
    UnsignedLessThanOrEqual,
    Overflow,
    NotOverflow,
}

pub fn get_cond_name(cmp: String) -> String {
    let cond = match cmp.as_ref() {
        "eq" => "IntCC::Equal".to_string(),
        "ne" => "IntCC::NotEqual".to_string(),
        "slt" => "IntCC::SignedLessThan".to_string(),
        "ult" => "IntCC::UnsignedLessThan".to_string(),
        "sle" => "IntCC::SignedLessThanOrEqual".to_string(),
        "ule" => "IntCC::UnsignedLessThanOrEqual".to_string(),
        // Souper does not generated ygt, sgt, sge, uge,
        // overflow, not overflow - conditions
        _ => "".to_string(),
    };
    cond
}

pub fn generate_baseline_matcher(mut nodes: Vec<Node>,
                                 mut rhs: HashMap<usize,
                                 Vec<CtonInst>>,
                                 mut count: u32) -> String {
    let mut opt_func = Opt::new();
    let mut arg_str = String::from("");
    let mut action_flag = false;
    let mut arg_counter : u32 = 0;
    let mut const_counter: u32 = 0;

    // Create and insert root node at the beginning of
    // vector of LHS single tree nodes
    nodes.insert(0, opt_func.build_root_node());

    for node in 0 .. nodes.len() {
        action_flag = is_node_actionable(nodes[node].id, rhs.clone());
        // dump: begin
        println!("Node ==== ======================");
        println!("\t\t Actionable? = {}", action_flag);
        println!("\t\t Node Type = {}",
                  lhspatternmatcher::get_node_type(
                  nodes[node].clone().node_type));
        println!("\t\t Node Id = {}", nodes[node].id);
        println!("\t\t Node Level = {}", nodes[node].level);
        println!("\t\t Node Value = {}", nodes[node].node_value);
        match nodes[node].next.clone() {
            Some(ids) => {
                for i in 0 .. ids.len() {
                    println!("\t\t Node->next = {}", ids[i].index);
                }
            },
            None => {
                println!("No next\n")
            },
        }
        // dump: end
        match nodes[node].node_type {
            NodeType::match_root => {
                opt_func.generate_header(count);
                let current_level = nodes[node].level;
                opt_func.enter_scope(ScopeType::scope_func,
                                     current_level);
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_instdata => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
               
                let mut opt_clone = opt_func.clone();
                let mut ent = opt_clone.current_entity;
                if !ent.is_empty() {
                    opt_func.append(String::from("match pos.func.dfg"));
                    opt_func.append(String::from("["));
                    // FIXME: Connect this ent string with RHS replacement part
                    opt_func.append(ent);
                    opt_func.append(String::from("]"));
                    opt_func.enter_scope(ScopeType::scope_match, current_level);
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::inst_type => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                // Check if there is any child node already
                // matched at same level
                // If yes, pop and exit scope first, and
                // then enter into new matching case
                let index = opt_func.does_level_exist_in_stack(current_level);
                if index != 0 {
                    opt_func.pop_and_exit_scope_from(index);
                }
                match nodes[node].node_value.as_ref() {
                    "Var" => {},
                    "Binary" => {
                        // FIXME: "args" part, make a connection
                        // between actual args and string
                        opt_func.append(String::from(
                                 "InstructionData::Binary { opcode, args }"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("opcode"));
                       // FIXED: Generate: "let args_<counter> = args;"
                       opt_func.append(String::from("let args_"));
                       arg_counter = opt_func.get_argument_counter(arg_counter);
                       opt_func.append(String::from(" = args;\n"));
                    },
                    "IntCompare" => {
                        // FIXME: "args" part, make a connection
                        // between actual args and string
                        opt_func.append(String::from(
                                 "InstructionData::IntCompare { opcode, cond, args }"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("opcode"));
                       // FIXED: Generate: "let args_<counter> = args;"
                       opt_func.append(String::from("let args_"));
                       arg_counter = opt_func.get_argument_counter(arg_counter);
                       opt_func.append(String::from(" = args;\n"));
                    },
                    "Unary" => {
                        // FIXME: "arg" part, make a connection
                        // b/w actual args and string
                        opt_func.append(String::from(
                                 "InstructionData::Unary { opcode, arg }"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("opcode"));
                       // FIXED: Generate: "let args_<counter> = arg;"
                       opt_func.append(String::from("let args_"));
                       arg_counter = opt_func.get_argument_counter(arg_counter);
                       opt_func.append(String::from(" = arg;\n"));
                    },
                    "BinaryImm" => {
                        // FIXME: "args" part, make a connection
                        // between actual args and string
                        opt_func.append(String::from(
                                 "InstructionData::BinaryImm { opcode, arg, imm }"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("opcode"));
                        opt_func.append(String::from("let args_"));
                        arg_counter = opt_func.get_argument_counter(arg_counter);
                        opt_func.append(String::from(" = arg;\n"));
                        // Push the rhs_<count> to ConstStack
                        const_counter = opt_func.get_const_counter(const_counter);
                        let mut rhs_arg = "rhs_".to_string();
                        rhs_arg.push_str(&const_counter.to_string());
                        opt_func.append(String::from("let "));
                        opt_func.append(String::from(rhs_arg.to_string()));
                        opt_func.append(String::from(" : i64 = imm.into();\n"));
                        opt_func.push_to_const_stack(rhs_arg.to_string());
                    },
                    "IntCompareImm" => {
                        // FIXME: "args" part, make a connection
                        // between actual args and string
                        opt_func.append(String::from(
                                 "InstructionData::IntCompareImm { opcode, cond, arg, imm }"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("opcode"));
                        opt_func.append(String::from("let args_"));
                        arg_counter = opt_func.get_argument_counter(arg_counter);
                        opt_func.append(String::from(" = arg;\n"));
                        // Push the rhs_<count> to ConstStack
                        const_counter = opt_func.get_const_counter(const_counter);
                        let mut rhs_arg = "rhs_".to_string();
                        rhs_arg.push_str(&const_counter.to_string());
                        opt_func.append(String::from("let "));
                        opt_func.append(String::from(rhs_arg.to_string()));
                        opt_func.append(String::from(" : i64 = imm.into();\n"));
                        opt_func.push_to_const_stack(rhs_arg.to_string());
                    },
                    _ => {
                        panic!("Error: This instruction data type is not yet handled");
                    },
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_valdef => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                // Check if there is any child node already
                // matched at same level
                // If yes, pop and exit scope first, and
                // then enter into new matching case
                let index = opt_func.does_level_exist_in_stack(current_level);
                if index != 0 {
                    opt_func.pop_and_exit_scope_from(index);
                }
                match nodes[node].node_value.as_ref() {
                    "Param" => {
                        // Reset the argument match string to empty str
                        // so that for further args, it's not appended.
                        arg_str = String::from("");
                        opt_func.set_entity(String::from(""));
                    },
                    "Result" => {
                        // for Result type, we want to generate arg match part,
                        // so, append it.
                        opt_func.append(arg_str.clone());
                        arg_str = String::from("");
                        opt_func.enter_scope(
                                 ScopeType::scope_match,
                                 current_level-1);
                        opt_func.append(String::from("\nValueDef::"));
                        opt_func.append(String::from("Result(arg_ty, _)"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                        opt_func.set_entity(String::from("arg_ty"));
                    },
                    _ => {
                        // FIXME - do we want error handling here
                        // for NoneType and ""
                        println!("\t\t entering unknown valdef case\n");
                    },
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_opcode => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                let mut opt_clone = opt_func.clone();
                let mut ent = opt_clone.current_entity;
                // FIXME: Any purpose of ent here?
                if !ent.is_empty() {
                    opt_func.append(String::from("match opcode"));
                    opt_func.enter_scope(
                             ScopeType::scope_match,
                             current_level);
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = rhs.get(&nodes[node].id);
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::opcode => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                // Check if there is any child node already
                // matched at same level
                // If yes, pop and exit scope first, and
                // then enter into new matching case
                let index = opt_func.does_level_exist_in_stack(current_level);
                if index != 0 {
                    opt_func.pop_and_exit_scope_from(index);
                }
                // match the actual opcode types
                match nodes[node].node_value.as_ref() {
                    "Var" => {},
                    "iadd" => {
                        opt_func.append(String::from("Opcode::Iadd"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "iadd_imm" => {
                        opt_func.append(String::from("Opcode::IaddImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "imul" => {
                        opt_func.append(String::from("Opcode::Imul"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "imul_imm" => {
                        opt_func.append(String::from("Opcode::ImulImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "isub" => {
                        opt_func.append(String::from("Opcode::Isub"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "irsub_imm" => {
                        opt_func.append(String::from("Opcode::IsubImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "icmp" => {
                        opt_func.append(String::from("Opcode::Icmp"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "icmp_imm" => {
                        opt_func.append(String::from("Opcode::IcmpImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "band" => {
                        opt_func.append(String::from("Opcode::Band"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "band_imm" => {
                        opt_func.append(String::from("Opcode::BandImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "bor" => {
                        opt_func.append(String::from("Opcode::Bor"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "bor_imm" => {
                        opt_func.append(String::from("Opcode::BorImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "bxor" => {
                        opt_func.append(String::from("Opcode::Bxor"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "bxor_imm" => {
                        opt_func.append(String::from("Opcode::BxorImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "ishl" => {
                        opt_func.append(String::from("Opcode::Ishl"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "ishl_imm" => {
                        opt_func.append(String::from("Opcode::IshlImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "sshr" => {
                        opt_func.append(String::from("Opcode::Sshr"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "sshr_imm" => {
                        opt_func.append(String::from("Opcode::SshrImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "ushr" => {
                        opt_func.append(String::from("Opcode::Ushr"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "ushr_imm" => {
                        opt_func.append(String::from("Opcode::UshrImm"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "popcnt" => {
                        opt_func.append(String::from("Opcode::Popcnt"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "clz" => {
                        opt_func.append(String::from("Opcode::Clz"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    "ctz" => {
                        opt_func.append(String::from("Opcode::Ctz"));
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    _ => {
                        panic!("Error: this opcode type is not yet handled");
                    },
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_cond => {
                let current_level = nodes[node].level;
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                let mut opt_clone = opt_func.clone();
                let mut ent = opt_clone.current_entity;
                // FIXME: Any purpose of ent here?
                if !ent.is_empty() {
                    opt_func.append(String::from("match cond"));
                    opt_func.enter_scope(
                             ScopeType::scope_match,
                             current_level);
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = rhs.get(&nodes[node].id);
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::cond => {
                let current_level = nodes[node].level;
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                // Check if there is any child node already
                // matched at same level
                // If yes, pop and exit scope first, and
                // then enter into new matching case
                let index = opt_func.does_level_exist_in_stack(current_level);
                if index != 0 {
                    opt_func.pop_and_exit_scope_from(index);
                }
                // match the actual opcode types
                match nodes[node].node_value.as_ref() {
                    "eq" | "ne" | "ult" |
                    "ule" | "slt" | "sle" => {
                        let cond = get_cond_name(
                                   nodes[node].clone().node_value);
                        opt_func.append(cond);
                        opt_func.enter_scope(
                                 ScopeType::scope_case,
                                 current_level);
                    },
                    _ => {
                        panic!("Error: this condition type is not yet handled");
                    },
                }
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_args => {
                let current_level = nodes[node].level;
                //set the level of root->next nodes to 0+1
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                // Create an optional argument matching string here
                // we will decide later whether we need this match
                // on args or not depending on if the argument type
                // is Result or Param. Param
                // type does not need this match part at all.
                let mut optional_argstr = String::from("");
                optional_argstr.push_str(&(String::from(
                                "match pos.func.dfg.value_def")));
                optional_argstr.push_str(&(String::from("(")));
                // make string like: args_2 or args_2[0]
                // depending on binaryImm or binary
                let arg_node_val = nodes[node].node_value.clone();
                optional_argstr.push_str(&(String::from("args_")));
                optional_argstr.push_str(
                                &(String::from(arg_counter.to_string())));
                if let Some(i) = arg_node_val.find('[') {
                    optional_argstr.push_str(
                                    &(nodes[node].node_value.clone())[i..]);
                }
                optional_argstr.push_str(&(String::from(")")));
                // FIXME: Do we want to take action here and should we
                // append to arg_str, or opt_func?
                if arg_str != optional_argstr {
                    arg_str.push_str(&optional_argstr);
                }
            },
            NodeType::match_plain_const => {
                let current_level = nodes[node].level;
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            NodeType::match_const => {
                let current_level = nodes[node].level;
                // FIXED: Earlier, I assumed that constant node will
                // mark an end of the instruction i.e. it will always be
                // second operand of immediate instruction, and hence there
                // were no setting of levels for next nodes.
                // But, const node can be the first operand as well, so
                // no matter what, we should always look up for next nodes
                // and set their levels.
                opt_func.set_level_of_all_child_nodes(
                         &mut nodes,
                         node,
                         current_level);
                let index = opt_func.does_level_exist_in_stack(current_level);
                if index != 0 {
                    opt_func.pop_and_exit_scope_from(index);
                }
                let const_value = &nodes[node].node_value;
                // FIXME: fix width of the constant in rhs part
                // Check Cranelift's instructions specifications
                const_counter = opt_func.get_const_counter(const_counter);
                // FIXME: pop the rhs immediate arguments from the ConstStack
                let rhs_arg = opt_func.pop_from_const_stack();
                opt_func.append(String::from("if "));
                opt_func.append(String::from(rhs_arg.to_string()));
                opt_func.append(String::from(" == "));
                opt_func.append(const_value.to_string());
                opt_func.enter_scope(
                         ScopeType::scope_func,
                         current_level);
                if action_flag {
                    action_flag = false;
                    let found_rhs = &rhs[&nodes[node].id];
                    opt_func.take_action(found_rhs.to_vec());
                }
            },
            _ => {
                panic!("\n\nmatch type not handled yet!\n");
            },
        }
    }

    // exit func scope
    // debug scope stack info
    //println!("********* Scope Stack ***********");
    //for x in 0 .. opt_func.scope_stack.len() {
    //    let elem = opt_func.scope_stack[x].clone();
    //    println!("Level of scope elem = {}", elem.level);
    //    match elem.scope_type {
    //        ScopeType::scope_func => {
    //            println!("scope func");
    //        },
    //        ScopeType::scope_match => {
    //            println!("scope match");
    //        },
    //        ScopeType::scope_case => {
    //            println!("scope case");
    //        },
    //        _ => {},
    //    }
    //}
    //println!("********* Scope Stack End ***********");
    for s in 0 .. opt_func.scope_stack.len() {
        match opt_func.scope_stack.pop() {
            Some(elem) => {
                opt_func.exit_scope(
                         elem.scope_type,
                         elem.level);
            },
            None => {},
        }
        //let elem_ty = opt_func.scope_stack.pop();
        //match elem_ty {
        //    Some(ty) => {
        //        opt_func.exit_scope(ty);
        //    },
        //    None => {},
        //}
    }

    opt_func.func_str
}
