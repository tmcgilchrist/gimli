#![feature(test)]

extern crate gimli;
extern crate test;

use gimli::{DebugAbbrev, DebugInfo, DebugLine, DebugLineOffset, LineNumberProgramHeader,
            LittleEndian, StateMachine};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn read_section(section: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into()));
    path.push("./fixtures/self/");
    path.push(section);

    assert!(path.is_file());
    let mut file = File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

#[bench]
fn bench_parsing_debug_abbrev(b: &mut test::Bencher) {
    let debug_info = read_section("debug_info");
    let debug_info = DebugInfo::<LittleEndian>::new(&debug_info);
    let unit = debug_info.units()
        .next()
        .expect("Should have at least one compilation unit")
        .expect("And it should parse OK");

    let debug_abbrev = read_section("debug_abbrev");

    b.iter(|| {
        let debug_abbrev = DebugAbbrev::<LittleEndian>::new(&debug_abbrev);
        test::black_box(unit.abbreviations(debug_abbrev)
            .expect("Should parse abbreviations"));
    });
}

#[bench]
fn bench_parsing_debug_info(b: &mut test::Bencher) {
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = DebugAbbrev::<LittleEndian>::new(&debug_abbrev);

    let debug_info = read_section("debug_info");

    b.iter(|| {
        let debug_info = DebugInfo::<LittleEndian>::new(&debug_info);

        for unit in debug_info.units() {
            let unit = unit.expect("Should parse compilation unit");
            let abbrevs = unit.abbreviations(debug_abbrev)
                .expect("Should parse abbreviations");

            let mut cursor = unit.entries(&abbrevs);
            while let Some((_, entry)) = cursor.next_dfs().expect("Should parse next dfs") {
                let mut attrs = entry.attrs();
                while let Some(attr) = attrs.next().expect("Should parse entry's attribute") {
                    test::black_box(&attr);
                }
            }
        }
    });
}

#[bench]
fn bench_parsing_line_number_program_opcodes(b: &mut test::Bencher) {
    let debug_line = read_section("debug_line");
    let debug_line = DebugLine::<LittleEndian>::new(&debug_line);

    // We happen to know that there is a line number program and header at
    // offset 0 and that address size is 8 bytes. No need to parse DIEs to grab
    // this info off of the compilation units.
    let offset = DebugLineOffset(0);
    let address_size = 8;

    b.iter(|| {
        let header = LineNumberProgramHeader::new(debug_line, offset, address_size)
            .expect("Should parse line number program header");

        for opcode in header.opcodes() {
            let opcode = opcode.expect("Should parse opcode");
            test::black_box(opcode);
        }
    });
}

#[bench]
fn bench_executing_line_number_programs(b: &mut test::Bencher) {
    let debug_line = read_section("debug_line");
    let debug_line = DebugLine::<LittleEndian>::new(&debug_line);

    // We happen to know that there is a line number program and header at
    // offset 0 and that address size is 8 bytes. No need to parse DIEs to grab
    // this info off of the compilation units.
    let offset = DebugLineOffset(0);
    let address_size = 8;

    b.iter(|| {
        let header = LineNumberProgramHeader::new(debug_line, offset, address_size)
            .expect("Should parse line number program header");

        let state_machine = StateMachine::new(&header);
        for row in state_machine {
            let row = row.expect("Should parse and execute all rows in the line number program");
            test::black_box(row);
        }
    });
}
