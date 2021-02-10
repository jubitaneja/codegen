// LHS Pattern matcher

use cliftinstbuilder::{self, CtonInst, CtonInstKind, CtonOpcode, CtonOperand, CtonValueDef};

pub struct Arena {
    nodes: Vec<Node>,
    clift_insts: Vec<CtonInst>,
    count: usize,
    instdata_count: usize,
}

#[derive(Clone)]
pub enum NodeType {
    MatchInstData,
    InstType,
    MatchOpcode,
    Opcode,
    MatchArgs,
    MatchValDef,
    MatchConst,
    MatchPlainConst,
    MatchRoot,
    MatchCond,
    Cond,
    MatchNone,
}

#[derive(Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub node_value: String,
    pub width: u32,
    pub id: usize,
    pub var_id: Option<u32>,
    pub arg_flag: bool,
    pub level: usize,
    pub next: Option<Vec<NodeID>>,
    pub idx_num: Option<usize>,
    pub arg_name: String,
}

#[derive(Clone)]
pub struct NodeID {
    pub index: usize,
}

#[derive(Clone)]
pub struct NodeIndex {
    node: Node,
    index: usize,
}

/// Helper functions
pub fn get_arg_name_for_binary_imm(_index: usize, argtype: &str) -> String {
    match argtype.as_ref() {
        "index" => "arg".to_string(),
        "const" => "imm".to_string(),
        _ => panic!("BinaryImm inst can only have operands index 0|1\n"),
    }
}

pub fn get_arg_name(index: usize) -> String {
    let mut arg = "args".to_string();
    arg.push_str("[");
    arg.push_str(&(index.to_string()));
    arg.push_str("]");
    arg
}

/// Returns the infer instruction from cranelift instrictions set
pub fn get_infer_clift_inst(clift_insts: Vec<CtonInst>) -> CtonInst {
    let mut infer_inst = CtonInst {
        valuedef: CtonValueDef::NoneType,
        kind: CtonInstKind::NoneType,
        opcode: CtonOpcode::NoneType,
        cond: None,
        width: 0,
        var_num: Some(0),
        cops: None,
    };
    for inst in clift_insts {
        match inst.opcode {
            CtonOpcode::Infer => {
                infer_inst = inst;
                break;
            }
            _ => {
                continue;
            }
        }
    }
    infer_inst
}

/// Returns the operands of infer instruction
pub fn get_infer_clift_op(infer_inst: CtonInst) -> Vec<CtonOperand> {
    let infer_ops = infer_inst.cops;
    match infer_ops {
        Some(ops) => {
            assert_eq!(ops.len(), 1, "Infer instruction must have only one operand");
            ops
        }
        None => panic!("Infer instruction must have one operand"),
    }
}

/// Returns the index of instruction where infer instruction points to
pub fn get_index_from_infer_clift_op(infer_ops: Vec<CtonOperand>) -> usize {
    let mut idx: usize = 0;
    for op in infer_ops {
        assert_eq!(
            op.const_val, None,
            "operand of infer inst must be an index to another inst"
        );
        match op.idx_val {
            Some(index) => {
                idx = index;
            }
            None => panic!("operand of infer inst must have a valid index value to another inst"),
        }
    }
    idx
}

pub fn get_total_number_of_args(inst: &CtonInst) -> usize {
    let ops = inst.cops.clone();
    match ops {
        Some(ops_list) => ops_list.len(),
        None => 0,
    }
}

pub fn get_node_type(ty: NodeType) -> String {
    match ty {
        NodeType::MatchInstData => "MatchInstData".to_string(),
        NodeType::InstType => "InstType".to_string(),
        NodeType::MatchOpcode => "MatchOpcode".to_string(),
        NodeType::Opcode => "Opcode".to_string(),
        NodeType::MatchArgs => "MatchArgs".to_string(),
        NodeType::MatchConst => "MatchConst".to_string(),
        NodeType::MatchPlainConst => "MatchPlainConst".to_string(),
        NodeType::MatchRoot => "MatchRoot".to_string(),
        NodeType::MatchValDef => "MatchValDef".to_string(),
        NodeType::MatchCond => "MatchCond".to_string(),
        NodeType::Cond => "Cond".to_string(),
        NodeType::MatchNone => panic!("Unexpected node type"),
    }
}

impl Arena {
    pub fn new(global_counter: usize) -> Arena {
        Arena {
            nodes: Vec::new(),
            clift_insts: Vec::new(),
            count: global_counter,
            instdata_count: 0,
        }
    }

    pub fn update_count(&mut self) {
        self.count += 1;
    }

    pub fn build_default_node(&mut self) -> Node {
        Node {
            node_type: NodeType::MatchNone,
            node_value: "".to_string(),
            width: 0,
            id: self.count,
            var_id: None,
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn build_instdata_node(&mut self, clift_inst: &CtonInst) -> Node {
        Node {
            node_type: NodeType::MatchInstData,
            node_value: "instdata".to_string(),
            width: 0,
            id: self.count,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn get_arg_name_for_instdata_node(&mut self, kind: CtonInstKind) -> String {
        let mut arg_name = "".to_string();
        match kind {
            // FIXME: Bug in this code for superopt_2 func in cranelift repo. case: 0 == 0?
            CtonInstKind::Unary | CtonInstKind::UnaryImm |
            CtonInstKind::Binary | CtonInstKind::BinaryImm |
            CtonInstKind::IntCompare | CtonInstKind::IntCompareImm => {
                arg_name.push_str("arg_");
                arg_name.push_str(&self.instdata_count.to_string());
                self.instdata_count += 1;
            },
            _ => {},
        }
        arg_name
    }

    pub fn build_specific_instdata_node(&mut self, clift_inst: &CtonInst) -> Node {
        let instdata_val = clift_inst.kind.clone();
        Node {
            node_type: NodeType::InstType,
            node_value: cliftinstbuilder::get_clift_instdata_name(instdata_val.clone()),
            width: clift_inst.width.clone(),
            id: self.count,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: self.get_arg_name_for_instdata_node(instdata_val.clone()),
        }
    }

    pub fn build_opcode_node(&mut self, clift_inst: &CtonInst) -> Node {
        Node {
            node_type: NodeType::MatchOpcode,
            node_value: "opcode".to_string(),
            id: self.count,
            width: 0,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn build_specific_opcode_node(&mut self, clift_inst: &CtonInst) -> Node {
        let width_val = clift_inst.width.clone();
        Node {
            node_type: NodeType::Opcode,
            node_value: cliftinstbuilder::get_clift_opcode_name(clift_inst.opcode.clone()),
            width: width_val,
            id: self.count,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn build_cond_node(&mut self, clift_inst: &CtonInst) -> Node {
        Node {
            node_type: NodeType::MatchCond,
            node_value: "cond".to_string(),
            id: self.count,
            width: 0,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn build_specific_cond_node(&mut self, clift_inst: &CtonInst) -> Node {
        let cond_val = clift_inst.cond.clone();
        let width_val = clift_inst.width.clone();
        Node {
            node_type: NodeType::Cond,
            node_value: cliftinstbuilder::get_clift_cond_name(cond_val),
            width: width_val,
            id: self.count,
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn set_next_of_prev_node(&mut self, current: Node, mut previous: Node) -> Node {
        let mut next_ids: Vec<NodeID> = Vec::new();
        next_ids.push(NodeID { index: current.id });
        previous.next = Some(next_ids);
        previous
    }

    pub fn set_next_of_current_node_by_default(&mut self, mut current: Node) -> Node {
        let mut next_ids: Vec<NodeID> = Vec::new();
        next_ids.push(NodeID {
            index: self.count.clone(),
        });
        current.next = Some(next_ids);
        current
    }

    pub fn build_separate_arg_node(
        &mut self,
        argtype: String,
        arg: usize,
        parent_instdata: String,
        idx_num: Option<usize>,
    ) -> Node {
        let node_val = match parent_instdata.as_ref() {
            "BinaryImm" | "IntCompareImm" | "UnaryImm" => get_arg_name_for_binary_imm(arg, &argtype),
            _ => {
                // Binary, Var, Unary
                get_arg_name(arg)
            }
        };
        Node {
            node_type: NodeType::MatchArgs,
            node_value: node_val,
            width: 0,
            id: self.count,
            var_id: None,
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: idx_num,
            arg_name: "".to_string(),
        }
    }

    pub fn build_valdef_node(&mut self, clift_inst: &CtonInst) -> Node {
        let valdef = clift_inst.valuedef.clone();
        Node {
            node_type: NodeType::MatchValDef,
            node_value: cliftinstbuilder::get_clift_valdef_name(valdef),
            id: self.count,
            width: clift_inst.width.clone(),
            var_id: clift_inst.var_num.clone(),
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    // FIXME: fix constant width to i64, maybe? depending on const value
    // width in SouperOperand and CtonOperand
    pub fn build_constant_node(&mut self, constant: i128) -> Node {
        // FIXME: Fix the width of constant
        Node {
            node_type: NodeType::MatchConst,
            node_value: constant.to_string(),
            id: self.count,
            width: 0,
            var_id: None,
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn build_plain_constant_node(&mut self) -> Node {
        // FIXME: Fix the width of constant
        Node {
            node_type: NodeType::MatchPlainConst,
            node_value: "constant".to_string(),
            id: self.count,
            width: 0,
            var_id: None,
            arg_flag: false,
            level: 0,
            next: None,
            idx_num: None,
            arg_name: "".to_string(),
        }
    }

    pub fn get_node_with_id(&mut self, idx: usize) -> Option<NodeIndex> {
        let mut ret_node = None;
        for n in 0..self.nodes.clone().len() {
            let node_id = self.nodes[n].clone().id;
            if node_id == idx {
                ret_node = Some(NodeIndex {
                    node: self.nodes[n].clone(),
                    index: n,
                });
            } else {
                ret_node = None;
            }
        }
        ret_node
    }

    pub fn get_clift_op_type_from_arg_num(
        &mut self,
        clift_inst: &CtonInst,
        arg_num: usize,
    ) -> String {
        let mut list_ops = Vec::new();
        let cops = &clift_inst.cops;
        if let Some(ops) = cops {
            for op in ops {
                if op.idx_val.is_some() {
                    println!("op is an index");
                    list_ops.push("index");
                }
                if op.const_val.is_some() {
                    list_ops.push("const");
                    match op.const_val {
                        Some(c) => println!("Op is a constant with value: {}\n", c),
                        _ => {},
                    }
                }
            }
        }
        list_ops[arg_num].to_string()
    }

    pub fn get_clift_op_index_num_from_arg_num(
        &mut self,
        clift_inst: &CtonInst,
        arg_num: usize,
    ) -> Option<usize> {
        println!("fn: get_clift_op_index_num_from_arg_num()");
        let cops = &clift_inst.cops;
        let mut idx = None;
        if let Some(ops) = cops {
            for op in 0..ops.len() {
                println!("\tfor op = {}, argnum = {}", op, arg_num);
                if op == arg_num {
                    match ops[op].idx_val {
                        Some(i) => {
                            println!("\t\tidx = {}", i);
                            idx = Some(i);
                        },
                        None => {},
                    }
                }
            }
        }
        idx
    }

    pub fn build_args_node(&mut self, clift_inst: &CtonInst, parent_instdata: String) {
        let total_args = get_total_number_of_args(clift_inst);
        for op in 0..total_args {
            let op_type = self.get_clift_op_type_from_arg_num(clift_inst, op);
            let idx_number = self.get_clift_op_index_num_from_arg_num(clift_inst, op);
            let named_arg_node = self.build_separate_arg_node(op_type, op, parent_instdata.clone(), idx_number);

            //set next of node before named arg node
            let c = self.count.clone();
            let node_x = self.get_node_with_id(c - 1);

            match node_x {
                Some(NodeIndex { mut node, index }) => {
                    let mut next_ids: Vec<NodeID> = Vec::new();
                    next_ids.push(NodeID { index: self.count });
                    node.next = Some(next_ids);
                    self.nodes[index] = node;
                }
                None => {
                    panic!("No node with the id = {} found in arena", self.count - 1);
                }
            }

            self.update_count();

            // FIXME: valdef node has to be constructed
            // while traversing the ops types
            // not the instruction types
            // take it down to clift_inst.cops part

            let mut arg_valdef_node = self.build_default_node();
            let cops = clift_inst.cops.clone();

            match cops.clone() {
                Some(ops) => {
                    let arg = &ops[op];
                    match arg.idx_val.clone() {
                        Some(idx) => {
                            let root_inst = &self.clift_insts[idx].clone();
                            arg_valdef_node = self.build_valdef_node(root_inst);
                        }
                        None => {
                            assert!(
                                arg.const_val.is_some(),
                                "clift inst op must have either an idx or const value"
                            );

                            // TODO: deal with constants later
                            // here if valdef is diff. for consts

                            // Build just a named "constant" node, later
                            // build a const node with const value in it.
                            arg_valdef_node = self.build_plain_constant_node();
                        }
                    }
                }
                None => {
                    println!("Expected args of an inst here\n");
                }
            }
            //let arg_valdef_node = self.build_valdef_node(clift_inst);

            // set the connection b/w above two nodes
            let updated_named_arg_node =
                self.set_next_of_prev_node(arg_valdef_node.clone(), named_arg_node.clone());

            self.update_count();
            // set next of valdef node because it's
            // sure to have some nodes after this
            let updated_valdef_node =
                self.set_next_of_current_node_by_default(arg_valdef_node.clone());

            self.nodes.push(updated_named_arg_node);
            self.nodes.push(updated_valdef_node);

            // repeat the prefix tree build here again!
            //let cops = clift_inst.cops.clone();
            match cops {
                Some(ops) => {
                    let arg = &ops[op];
                    match arg.idx_val.clone() {
                        Some(idx) => {
                            let root_inst = &self.clift_insts[idx].clone();
                            self.build_sequence_of_nodes(root_inst);
                        },
                        None => {
                            match arg.const_val.clone() {
                                Some(constant) => {
                                    let const_arg_node = self.build_constant_node(
                                                              constant);
                                    self.update_count();
                                    self.nodes.push(const_arg_node);
                                },
                                None => {
                                    panic!("operand of a clift inst must have either an index value or a constant value")
                                }
                            }
                        },
                    }
                }
                None => {
                    panic!(
                        "The clift instruction is expected to have {} operands",
                        total_args
                    );
                }
            }
        }
    }

    pub fn build_sequence_of_nodes(&mut self, clift_inst: &CtonInst) -> Vec<Node> {
        let node_instdata = self.build_instdata_node(clift_inst);
        self.update_count();

        let node_specific_inst = self.build_specific_instdata_node(clift_inst);
        self.update_count();

        let updated_instdata =
            self.set_next_of_prev_node(node_specific_inst.clone(), node_instdata.clone());

        let node_opcode = self.build_opcode_node(clift_inst);
        self.update_count();

        let updated_spec_inst =
            self.set_next_of_prev_node(node_opcode.clone(), node_specific_inst.clone());

        let node_specific_opcode = self.build_specific_opcode_node(clift_inst);
        self.update_count();

        let updated_opcode =
            self.set_next_of_prev_node(node_specific_opcode.clone(), node_opcode.clone());

        if node_specific_opcode.node_value.clone().contains("icmp") {
            let node_cond = self.build_cond_node(clift_inst);
            self.update_count();

            let updated_spec_opcode =
                self.set_next_of_prev_node(node_cond.clone(), node_specific_opcode.clone());

            let node_specific_cond = self.build_specific_cond_node(clift_inst);
            self.update_count();

            let updated_cond =
                self.set_next_of_prev_node(node_specific_cond.clone(), node_cond.clone());

            self.nodes.push(updated_instdata.clone());
            self.nodes.push(updated_spec_inst.clone());
            self.nodes.push(updated_opcode.clone());
            self.nodes.push(updated_spec_opcode.clone());
            self.nodes.push(updated_cond.clone());
            self.nodes.push(node_specific_cond.clone());
        } else {
            self.nodes.push(updated_instdata.clone());
            self.nodes.push(updated_spec_inst.clone());
            self.nodes.push(updated_opcode.clone());
            self.nodes.push(node_specific_opcode);
        }

        self.build_args_node(clift_inst, node_specific_inst.clone().node_value);

        self.nodes.clone()
    }
}

pub fn generate_single_tree_patterns(
    clift_insts: Vec<CtonInst>,
    global_count: usize
) -> Vec<Node> {
    let infer_clift_inst = get_infer_clift_inst(clift_insts.clone());
    let infer_clift_ops = get_infer_clift_op(infer_clift_inst);
    let index_from_infer_clift_op = get_index_from_infer_clift_op(infer_clift_ops);
    let inst_at_infer_op_idx = &clift_insts[index_from_infer_clift_op];

    // Create Arena and initialize it
    let mut arena = Arena::new(global_count);
    arena.clift_insts = clift_insts.clone();
    let all_nodes = arena.build_sequence_of_nodes(inst_at_infer_op_idx);

    println!("--- LHS pattern Matcher module: list of nodes -----------");
    for n in 0 .. all_nodes.len() {
        println!("Node id = {}", all_nodes[n].id);
        match all_nodes[n].clone().var_id {
            Some (var_num) => {
                println!("Var number = {}", var_num);
            },
            None => {
                println!("Var number = None\n");
            },
        }
        println!("Node type = {}", get_node_type(all_nodes[n].clone().node_type));
        println!("Node value = {}", all_nodes[n].node_value);
        match all_nodes[n].idx_num.clone() {
            Some(i) => println!("Node op idx num = {}", i),
            None => println!("Node op idx num = NONE"),
        }
        println!("Node arg_name == {}", all_nodes[n].arg_name);
        match all_nodes[n].clone().next {
            Some(x) => {
                for i in 0 .. x.len() {
                    println!("next = {}", x[i].index);
                }
            },
            None => {
                println!("next = None");
            }
        }
        println!("--------------------------------");
    }
    all_nodes
}
