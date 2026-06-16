use std::{fs::{self, File}, io::{BufRead, BufReader, Write}};

use serde_json::Value;

use crate::{bit_utils::bit_set, cpu::CPU};

pub mod bit_utils;

pub mod cpu;

pub mod instructions;

// AI-generated test runner. Sorry, not sorry!
fn run_test(name: &str) {
    let str = fs::read_to_string(name).unwrap();
    let tests: Value = serde_json::from_str(&str).unwrap();
    let tests = tests.as_array().unwrap();

    let total_tests = tests.len();
    let mut passed = 0;
    let mut failed = 0;

    for (index, test) in tests.iter().enumerate() {
        let mut cpu = CPU::new();

        let initial = &test["initial"];
        cpu.pc  = initial["pc"].as_u64().unwrap() as u16;
        cpu.sp  = initial["s"].as_u64().unwrap() as u8;
        cpu.p   = initial["p"].as_u64().unwrap() as u8;
        cpu.a   = initial["a"].as_u64().unwrap() as u8;
        cpu.x   = initial["x"].as_u64().unwrap() as u8;
        cpu.y   = initial["y"].as_u64().unwrap() as u8;
        
        let ram = initial["ram"].as_array().unwrap();
        for set in ram {
            let idx = set[0].as_u64().unwrap() as usize;
            let value = set[1].as_u64().unwrap() as u8;
            cpu.ram[idx] = value;
        }

        if !cpu.step() { 
            println!("[Test #{}] Unknown instruction; skipping test.", index);

            break;
        }

        let end = &test["final"]; 
        let mut test_failed = false;
        let mut errors = Vec::new();

        let mut check_reg = |actual: u64, expected: u64, name: &str| {
            if actual != expected {
                test_failed = true;
                errors.push(format!("  {} mismatch -> Got: {}, Expected: {}", name, actual, expected));
            }
        };

        check_reg(cpu.pc as u64, end["pc"].as_u64().unwrap(), "PC");
        check_reg(cpu.sp as u64, end["s"].as_u64().unwrap(), "SP");
        check_reg(cpu.p as u64, end["p"].as_u64().unwrap(), "Status (P)");
        check_reg(cpu.a as u64, end["a"].as_u64().unwrap(), "Accumulator (A)");
        check_reg(cpu.x as u64, end["x"].as_u64().unwrap(), "X Register");
        check_reg(cpu.y as u64, end["y"].as_u64().unwrap(), "Y Register");

        let end_ram = end["ram"].as_array().unwrap();
        for set in end_ram {
            let ram_idx = set[0].as_u64().unwrap() as usize;
            let expected_ram_val = set[1].as_u64().unwrap() as u8;

            if cpu.ram[ram_idx] != expected_ram_val {
                test_failed = true;
                errors.push(format!(
                    "  RAM mismatch at [{:04X}] -> Got: {}, Expected: {}", 
                    ram_idx, cpu.ram[ram_idx], expected_ram_val
                ));
            }
        }

        if test_failed {
            failed += 1;
            let test_id = test["name"].as_str().unwrap_or("Unnamed");
            println!("❌ Test #{} ({}) FAILED:", index, test_id);
            for err in errors {
                println!("{}", err);
            }
            println!("--------------------------------------------------");
        } else {
            passed += 1;
        }
    }

    println!("\n=== TEST RESULTS FOR {} ===", name);
    println!("Total Executed: {}", total_tests);
    println!("Passed: 🟩 {}", passed);
    println!("Failed: 🟥 {}", failed);

    if failed > 0 {
        panic!();
    }
}

fn main() {
    for i in 0x00..0xFF {
        println!("Instruction {:02x}", i);
        
        run_test(&format!("tests/single_step/{:02x}.json", i));
    }
    
}
