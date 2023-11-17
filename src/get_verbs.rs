pub mod parse_operand;
use crate::{
    asm_line::{AsmLine, CC},
    get_verbs::parse_operand::parse_operand,
    source_cursor::SourceCodeCursor,
};

pub fn get_tokens(source_code_contents: String) -> Vec<AsmLine> {
    let mut cursor = SourceCodeCursor::new(source_code_contents);

    let mut lines = Vec::new();

    while cursor.peek().is_some() {
        // this loop will consume one line per iteration:

        // consume leading whitespace
        consume_whitespace(&mut cursor);
        match cursor.peek() {
            None => break,
            Some('\n') | Some(';') | Some('.') => {
                // empty line or comment/directive. Consume the empty line.
                consume_rest_of_line(&mut cursor);
            }

            _ => {
                let mut component_1 = String::new();
                while cursor.peek().is_some() && !cursor.peek().unwrap().is_ascii_whitespace() {
                    let c = cursor.next().unwrap();
                    component_1.push(c);
                }

                if component_1.ends_with(":") {
                    // parse label
                    lines.push(AsmLine::Label(
                        component_1.strip_suffix(":").unwrap().to_owned(),
                    ));
                } else {
                    let component_1_base;
                    let mut is_byte_instr = false;
                    if component_1.ends_with(".W") || component_1.ends_with(".B") {
                        is_byte_instr = component_1.ends_with(".B");
                        component_1_base = &component_1[..component_1.len() - 2];
                    } else {
                        component_1_base = &component_1;
                    }

                    match component_1_base {
                        "RRC" | "SWPB" | "RRA" | "SXT" | "PUSH" | "CALL" => {
                            // SINGLE OPERAND FAMILY
                            consume_whitespace(&mut cursor);
                            let operand = parse_operand(&mut cursor);
                            match component_1_base {
                                "RRC" => {
                                    lines.push(AsmLine::RRC(operand, is_byte_instr));
                                }
                                "SWPB" => {
                                    lines.push(AsmLine::SWPB(operand));
                                }
                                "RRA" => {
                                    lines.push(AsmLine::RRA(operand, is_byte_instr));
                                }
                                "SXT" => {
                                    lines.push(AsmLine::SXT(operand));
                                }
                                "PUSH" => {
                                    lines.push(AsmLine::PUSH(operand, is_byte_instr));
                                }
                                "CALL" => {
                                    lines.push(AsmLine::CALL(operand));
                                }
                                _ => unreachable!(),
                            }
                        }
                        "RETI" => {
                            lines.push(AsmLine::RETI);
                        }

                        // JUMPS
                        "JNE" | "JNZ" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::NotEq, label));
                        }
                        "JEQ" | "JZ" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::Eq, label));
                        }
                        "JNC" | "JLO" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::NoCarry, label));
                        }
                        "JC" | "JHS" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::Carry, label));
                        }
                        "JN" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::Neg, label));
                        }
                        "JGE" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::GreaterEq, label));
                        }
                        "JL" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::Less, label));
                        }
                        "JMP" => {
                            let label = parse_jmp_label(&mut cursor);
                            lines.push(AsmLine::Jump(CC::Unconditional, label));
                        }

                        "MOV" | "ADD" | "ADDC" | "SUB" | "SUBC" | "CMP" | "DADD" | "BIT"
                        | "BIC" | "BIS" | "XOR" | "AND" => {
                            // DOUBLE OPERAND FAMILY
                            consume_whitespace(&mut cursor);
                            let operand_1 = parse_operand(&mut cursor);
                            assert_eq!(cursor.next(), Some(','));
                            let operand_2 = parse_operand(&mut cursor);
                            match component_1_base {
                                "MOV" => {
                                    lines.push(AsmLine::MOV(operand_1, operand_2, is_byte_instr));
                                }
                                "ADD" => {
                                    lines.push(AsmLine::ADD(operand_1, operand_2, is_byte_instr));
                                }
                                "ADDC" => {
                                    lines.push(AsmLine::ADDC(operand_1, operand_2, is_byte_instr));
                                }
                                "SUB" => {
                                    lines.push(AsmLine::SUB(operand_1, operand_2, is_byte_instr));
                                }
                                "SUBC" => {
                                    lines.push(AsmLine::SUBC(operand_1, operand_2, is_byte_instr));
                                }
                                "CMP" => {
                                    lines.push(AsmLine::CMP(operand_1, operand_2, is_byte_instr));
                                }
                                "DADD" => {
                                    lines.push(AsmLine::DADD(operand_1, operand_2, is_byte_instr));
                                }
                                "BIT" => {
                                    lines.push(AsmLine::BIT(operand_1, operand_2, is_byte_instr));
                                }
                                "BIC" => {
                                    lines.push(AsmLine::BIC(operand_1, operand_2, is_byte_instr));
                                }
                                "BIS" => {
                                    lines.push(AsmLine::BIS(operand_1, operand_2, is_byte_instr));
                                }
                                "XOR" => {
                                    lines.push(AsmLine::XOR(operand_1, operand_2, is_byte_instr));
                                }
                                "AND" => {
                                    lines.push(AsmLine::AND(operand_1, operand_2, is_byte_instr));
                                }
                                _ => unreachable!(),
                            }
                        }

                        _ => {
                            println!(
                                "warning: ignoring unrecognized instruction {}",
                                component_1_base
                            );
                        }
                    }
                }

                consume_rest_of_line(&mut cursor);
            }
        }
    }

    return lines;
}

fn parse_jmp_label(cursor: &mut SourceCodeCursor) -> String {
    consume_whitespace(cursor);
    let mut res = String::new();
    while cursor.peek().is_some() && !cursor.peek().unwrap().is_ascii_whitespace() {
        let c = cursor.next().unwrap();
        res.push(c);
    }
    consume_rest_of_line(cursor);

    res
}

pub fn consume_rest_of_line(cursor: &mut SourceCodeCursor) {
    while cursor.peek() != Some('\n') && cursor.peek() != None {
        cursor.next();
    }
    // consume newline if there is one
    cursor.next();
}

pub fn consume_whitespace(cursor: &mut SourceCodeCursor) {
    while cursor.peek() == Some(' ') || cursor.peek() == Some('\t') {
        cursor.next();
    }
}
