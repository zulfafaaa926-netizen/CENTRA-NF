//! IR — Intermediate Representation.
//!
//! Lowering from AST to deterministic instruction stream.
//! Same input AST → same instruction stream, always.

use crate::ast::ProcedureStatement;
use crate::ast::Program;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Compress {
        target: String,
    },
    VerifyIntegrity {
        target: String,
    },
    Transcode {
        target: String,
        output_type: String,
    },
    Filter {
        target: String,
        condition: String,
    },
    Aggregate {
        targets: Vec<String>,
        operation: String,
    },
    Convert {
        target: String,
        output_type: String,
    },
    Merge {
        targets: Vec<String>,
        output_name: String,
    },
    Split {
        target: String,
        parts: String,
    },
    Validate {
        target: String,
        schema: String,
    },
    Extract {
        target: String,
        path: String,
    },
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Compress { target } => {
                write!(f, "COMPRESS({})", target)
            }
            Instruction::VerifyIntegrity { target } => {
                write!(f, "VERIFY-INTEGRITY({})", target)
            }
            Instruction::Transcode {
                target,
                output_type,
            } => {
                write!(f, "TRANSCODE({} -> {})", target, output_type)
            }
            Instruction::Filter { target, condition } => {
                write!(f, "FILTER({} WHERE {})", target, condition)
            }
            Instruction::Aggregate { targets, operation } => {
                write!(f, "AGGREGATE({} AS {})", targets.join(","), operation)
            }
            Instruction::Convert {
                target,
                output_type,
            } => {
                write!(f, "CONVERT({} -> {})", target, output_type)
            }
            Instruction::Merge {
                targets,
                output_name,
            } => {
                write!(f, "MERGE({} INTO {})", targets.join(","), output_name)
            }
            Instruction::Split { target, parts } => {
                write!(f, "SPLIT({} INTO {} PARTS)", target, parts)
            }
            Instruction::Validate { target, schema } => {
                write!(f, "VALIDATE({} AGAINST {})", target, schema)
            }
            Instruction::Extract { target, path } => {
                write!(f, "EXTRACT({} FROM {})", path, target)
            }
        }
    }
}

pub fn lower(program: Program) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();

    // Validate that all procedure statements reference valid variables
    let declared_vars: std::collections::HashSet<String> = program
        .data
        .variables
        .iter()
        .map(|v| v.name.clone())
        .collect();

    for stmt in &program.procedure.statements {
        match stmt {
            ProcedureStatement::Compress { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Compress {
                    target: target.clone(),
                });
            }
            ProcedureStatement::VerifyIntegrity { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::VerifyIntegrity {
                    target: target.clone(),
                });
            }
            ProcedureStatement::Transcode {
                target,
                output_type,
            } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Transcode {
                    target: target.clone(),
                    output_type: output_type.to_string(),
                });
            }
            ProcedureStatement::Filter { target, condition } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Filter {
                    target: target.clone(),
                    condition: condition.clone(),
                });
            }
            ProcedureStatement::Aggregate { targets, operation } => {
                for target in targets {
                    if !declared_vars.contains(target) {
                        return Err(format!(
                            "Variable '{}' not declared in DATA DIVISION",
                            target
                        ));
                    }
                }
                instructions.push(Instruction::Aggregate {
                    targets: targets.clone(),
                    operation: operation.clone(),
                });
            }
            ProcedureStatement::Convert {
                target,
                output_type,
            } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Convert {
                    target: target.clone(),
                    output_type: output_type.to_string(),
                });
            }
            ProcedureStatement::Merge {
                targets,
                output_name,
            } => {
                for target in targets {
                    if !declared_vars.contains(target) {
                        return Err(format!(
                            "Variable '{}' not declared in DATA DIVISION",
                            target
                        ));
                    }
                }
                instructions.push(Instruction::Merge {
                    targets: targets.clone(),
                    output_name: output_name.clone(),
                });
            }
            ProcedureStatement::Split { target, parts } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Split {
                    target: target.clone(),
                    parts: parts.clone(),
                });
            }
            ProcedureStatement::Validate { target, schema } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Validate {
                    target: target.clone(),
                    schema: schema.clone(),
                });
            }
            ProcedureStatement::Extract { target, path } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Extract {
                    target: target.clone(),
                    path: path.clone(),
                });
            }
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_deterministic() {
        // Simple test that IR is deterministic
        let instr1 = Instruction::Compress {
            target: "buf".to_string(),
        };
        let instr2 = Instruction::Compress {
            target: "buf".to_string(),
        };
        assert_eq!(instr1, instr2);
    }
}
