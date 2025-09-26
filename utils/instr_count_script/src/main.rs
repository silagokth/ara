use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <filename> <function_name> <n_lanes> [<reg_name> <value> ...]",
            args[0]
        );
        std::process::exit(1);
    } else {
        println!("Number of arguments: {}", args.len());
        print!("{}", args[0]);
        for arg in &args {
            print!(" {}", arg);
        }
        println! {""};
    }
    let filename = &args[1];
    let function_name = &args[2];
    let n_lanes: i64 = args[3].parse().expect("Please provide a valid integer");
    let vlen: i64 = 1024 * n_lanes;
    println!("filename: {}", filename);
    println!("function_name: {}", function_name);
    println!("number of lanes: {}", n_lanes);
    println!("vlen: {}", vlen);

    // Get the register values
    let mut registers: HashMap<String, i64> = HashMap::new();
    for i in (4..args.len()).step_by(2) {
        let reg_name = &args[i];
        let reg_value: i64 = args[i + 1].parse().expect("Please provide a valid integer");
        println!("Register {}: {}", reg_name, reg_value);
        registers.insert(reg_name.to_string(), reg_value);
    }

    let mut memory: HashMap<i64, i64> = HashMap::new();

    // Read the file
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let mut index = 0;
    let mut first_line = None;
    let mut end_line = None;
    let mut total_instructions = 0;
    while first_line.is_none() || end_line.is_none() {
        let line = &all_lines[index];

        // Example of dynamically changing the line being read
        if line.contains("some_condition") {
            let new_line = "This is a new line content".to_string();
            println!("Modified Line {}: {}", index + 1, new_line);
        }
        // Check if start or end line
        if line.contains(format!("<{}>:", function_name).as_str()) {
            first_line = Some(index);
            println!("Function {} found at line {}", function_name, index + 1);
            index += 1;
            continue;
        } else if !first_line.is_none() && line.contains("\tret") {
            end_line = Some(index);
            println!("End of function found at line {}", index + 1);
            break;
        }

        // Interprete the line
        if !first_line.is_none() && end_line.is_none() {
            // Print the first word in col 25
            let opcode = &line[24..].split_whitespace().next().unwrap();
            let rest_of_line = &line[24..].split_whitespace().skip(1).collect::<Vec<&str>>();
            print!("Line {}: {} ", index + 1, opcode);

            total_instructions += 1;
            match opcode {
                &"vsetvli" => {
                    let vl_reg = rest_of_line[0].replace(&[',', ' '][..], "");
                    let avl_reg = rest_of_line[1].replace(&[',', ' '][..], "");
                    let avl: i64 = get_register_value(&mut registers, &avl_reg);
                    let sew: i64 = rest_of_line[2]
                        .replace(&[',', ' ', 'e'][..], "")
                        .parse::<i64>()
                        .unwrap();
                    let lmul = rest_of_line[3]
                        .replace(&[',', ' ', 'm'][..], "")
                        .parse::<i64>()
                        .unwrap();
                    let vlmax: i64 = vlen / sew;
                    let vl = std::cmp::min(avl, vlmax / lmul);
                    registers.insert(vl_reg.clone(), vl);

                    println!(
                        "avl: {}, sew: {}, lmul: {} => vlmax: {} => vl: {} (set to {})",
                        avl, sew, lmul, vlmax, vl, vl_reg
                    );
                }
                &"beqz" => {
                    let rs1 = rest_of_line[0].replace(&[',', ' '][..], "");
                    let address = rest_of_line[1]
                        .replace(&[',', ' '][..], "")
                        .replace("0x", "");
                    print!("{}({}) 0x{}", registers.get(&rs1).unwrap(), rs1, address);
                    if registers.get(&rs1).unwrap() == &0 {
                        let new_index = all_lines.iter().position(|l| {
                            l.replace(':', "").split_whitespace().next()
                                == Some(&address.to_string())
                        });
                        if let Some(new_index) = new_index {
                            index = new_index;
                            println!("(line {})", index + 1);
                            continue;
                        } else {
                            eprintln!("Address {} not found in the file", address);
                            std::process::exit(1);
                        }
                    } else if registers.get(&rs1).unwrap() < &0 {
                        eprintln!("rs1 is less than 0");
                        std::process::exit(1);
                    } else {
                        println!("");
                    }
                }
                &"mv" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let value = get_register_value(&mut registers, &rs1);
                    registers.insert(rd.clone(), value);
                    println!("{} {}({})", rd, value, rs1);
                }
                &"j" => {
                    let address = rest_of_line[0]
                        .replace(&[',', ' '][..], "")
                        .replace("0x", "");
                    let new_index = all_lines.iter().position(|l| {
                        l.replace(':', "").split_whitespace().next() == Some(&address.to_string())
                    });
                    if let Some(new_index) = new_index {
                        index = new_index;
                        println!("0x{} (line {})", address, index + 1);
                        continue;
                    } else {
                        eprintln!("Address {} not found in the file", address);
                        std::process::exit(1);
                    }
                }
                &"bne" => {
                    let rs1 = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs2 = rest_of_line[1].replace(&[',', ' '][..], "");
                    print!(
                        "{}({}) {}({})",
                        registers.get(&rs1).unwrap(),
                        rs1,
                        registers.get(&rs2).unwrap(),
                        rs2
                    );
                    if registers.get(&rs1).unwrap() != registers.get(&rs2).unwrap() {
                        let address = rest_of_line[2]
                            .replace(&[',', ' '][..], "")
                            .replace("0x", "");
                        print!(" 0x{}", address);
                        let new_index = all_lines.iter().position(|l| {
                            l.replace(':', "").split_whitespace().next()
                                == Some(&address.to_string())
                        });
                        if let Some(new_index) = new_index {
                            index = new_index;
                            println!("(line {})", index + 1);
                            continue;
                        } else {
                            eprintln!("Address {} not found in the file", address);
                            std::process::exit(1);
                        }
                    } else {
                        println!("");
                    }
                }
                &"slli" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let shamt = rest_of_line[2]
                        .replace(&[',', ' '][..], "")
                        .parse::<i64>()
                        .unwrap();
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let res_value = rs1_value << shamt;
                    println!("{} {}({}) << {} = {}", rd, rs1_value, rs1, shamt, res_value);
                    registers.insert(rd.clone(), res_value);
                }
                &"add" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let rs2 = rest_of_line[2].replace(&[',', ' '][..], "");
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let rs2_value = get_register_value(&mut registers, &rs2);
                    let res_value = rs1_value + rs2_value;
                    registers.insert(rd.clone(), res_value);
                    println!(
                        "{} {}({}) + {}({}) = {}({})",
                        rd, rs1_value, rs1, rs2_value, rs2, res_value, rd
                    );
                }
                &"sub" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let rs2 = rest_of_line[2].replace(&[',', ' '][..], "");
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let rs2_value = get_register_value(&mut registers, &rs2);
                    let res_value = rs1_value - rs2_value;
                    registers.insert(rd.clone(), res_value);
                    println!(
                        "{} {}({}) - {}({}) = {}({})",
                        rd, rs1_value, rs1, rs2_value, rs2, res_value, rd
                    );
                }
                &"addi" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let imm = rest_of_line[2]
                        .replace(&[',', ' '][..], "")
                        .parse::<i64>()
                        .unwrap();
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let res_value = rs1_value + imm;
                    registers.insert(rd.clone(), res_value);
                    println!(
                        "{} {}({}) + {} = {}({})",
                        rd, rs1_value, rs1, imm, res_value, rd
                    );
                }
                &"blt" => {
                    let rs1 = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs2 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let rs2_value = get_register_value(&mut registers, &rs2);
                    print!("{}({}) {}({})", rs1_value, rs1, rs2_value, rs2);
                    if rs1_value < rs2_value {
                        let address = rest_of_line[2]
                            .replace(&[',', ' '][..], "")
                            .replace("0x", "");
                        print!(" 0x{}", address);
                        let new_index = all_lines.iter().position(|l| {
                            l.replace(':', "").split_whitespace().next()
                                == Some(&address.to_string())
                        });
                        if let Some(new_index) = new_index {
                            index = new_index;
                            println!("(line {})", index + 1);
                            continue;
                        } else {
                            eprintln!("Address {} not found in the file", address);
                            std::process::exit(1);
                        }
                    } else {
                        println!("");
                    }
                }
                &"auipc" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let imm = rest_of_line[1]
                        .replace(&[',', ' '][..], "")
                        .replace("0x", "")
                        .parse::<i64>()
                        .unwrap()
                        << 12;
                    let pc_str = line.split_whitespace().next().unwrap().replace(":", "");
                    let pc = i64::from_str_radix(&pc_str, 16).unwrap();
                    let res_value = pc + imm;
                    registers.insert(rd.clone(), res_value);
                    println!("{} 0x{} + {} = {}({})", rd, index, imm, res_value, rd);
                }
                &"jalr" => {
                    // format = jalr imm(rd)
                    let pc_str = line.split_whitespace().next().unwrap().replace(":", "");
                    let pc = i64::from_str_radix(&pc_str, 16).unwrap();
                    let t = pc + 4;
                    let rd = rest_of_line[0]
                        .replace(&[',', ' '][..], "")
                        .split('(')
                        .collect::<Vec<&str>>()[1]
                        .replace(")", "");
                    let imm = rest_of_line[0]
                        .replace(&[',', ' '][..], "")
                        .split('(')
                        .collect::<Vec<&str>>()[0]
                        .parse::<i64>()
                        .unwrap();
                    let ra_value = get_register_value(&mut registers, &rd) + 4;
                    let res_value = (ra_value + imm) & !1;
                    registers.insert("ra".to_string(), t);

                    let jump_address = format!("{:x}", res_value);
                    let new_index = all_lines.iter().position(|l| {
                        l.replace(':', "").split_whitespace().next() == Some(&jump_address)
                    });
                    if let Some(new_index) = new_index {
                        index = new_index;
                        println!("(line {})", index + 1);
                        continue;
                    } else {
                        eprintln!(
                            "Address {} not found in the file ({:x}+{})",
                            jump_address, ra_value, imm
                        );
                        std::process::exit(1);
                    }
                }
                &"li" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let imm = rest_of_line[1]
                        .replace(&[',', ' '][..], "")
                        .parse::<i64>()
                        .unwrap();
                    registers.insert(rd.clone(), imm);
                    println!("{} {}({})", rd, imm, rd);
                }
                &"ld" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let offset = rest_of_line[1]
                        .replace(&[',', ' '][..], "")
                        .split('(')
                        .collect::<Vec<&str>>()[0]
                        .parse::<i64>()
                        .unwrap();
                    let rs1 = rest_of_line[1]
                        .replace(&[',', ' '][..], "")
                        .split('(')
                        .collect::<Vec<&str>>()[1]
                        .replace(")", "");
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let address = rs1_value + offset;
                    let value = get_memory_value(&mut memory, address);
                    registers.insert(rd.clone(), value);
                    println!(
                        "{} {}({})[0x{:x}] = {}({})",
                        rd, offset, rs1, address, value, rd
                    );
                }
                &"mul" => {
                    let rd = rest_of_line[0].replace(&[',', ' '][..], "");
                    let rs1 = rest_of_line[1].replace(&[',', ' '][..], "");
                    let rs2 = rest_of_line[2].replace(&[',', ' '][..], "");
                    let rs1_value = get_register_value(&mut registers, &rs1);
                    let rs2_value = get_register_value(&mut registers, &rs2);
                    let res_value = rs1_value * rs2_value;
                    registers.insert(rd.clone(), res_value);
                    println!(
                        "{} {}({}) * {}({}) = {}({})",
                        rd, rs1_value, rs1, rs2_value, rs2, res_value, rd
                    );
                }
                _ => {
                    println!("");
                }
            }
        }
        index += 1;
    }

    match (first_line, end_line) {
        (Some(first), Some(second)) => {
            println!("Number of lines between: {}", second - first - 1);
            println!("Total number of cycles: {}", total_instructions);
        }
        _ => {
            println!("The file does not contain two lines with 'csrwi 3054'");
        }
    }

    Ok(())
}

fn get_register_value(registers: &mut HashMap<String, i64>, reg_name: &str) -> i64 {
    if registers.contains_key(reg_name) {
        return *registers.get(reg_name).unwrap();
    } else {
        registers.entry(reg_name.to_string()).or_insert(0);
        return 0;
    }
}

fn get_memory_value(memory: &mut HashMap<i64, i64>, address: i64) -> i64 {
    if memory.contains_key(&address) {
        return *memory.get(&address).unwrap();
    } else {
        memory.entry(address).or_insert(0);
        return 0;
    }
}
