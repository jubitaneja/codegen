fn matcher(pos: &mut FuncCursor, inst: Inst) {
    match pos.func.dfg[inst] {
        InstructionData::Binary { opcode, args } => {
            match opcode {
                Opcode::Isub => {
                    match pos.func.dfg.val_def(args[0]) {
                        ValDef::Result(arg_ty, _) => {
                            match pos.func.dfg[arg_ty] {
                                InstructionData::Binary { opcode, args } => {
                                    match opcode {
                                        Opcode::Iadd => {
                                            match pos.func.dfg.val_def(args[0]) {
                                                ValDef::Param(_, _) => {
                                                    match pos.func.dfg.val_def(args[1]) {
                                                        ValDef::Param(_, _) => {
                                                            match pos.func.dfg.val_def(args[1]) {
                                                                ValDef::Param(_, _) => {
                                                                    // perform transformation
                                                                    unimplemented!();
                                                                },
                                                            }
                                                        },
                                                    }
                                                },
                                            }
                                        },
                                    }
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}