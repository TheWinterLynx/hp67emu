use std::fmt::Write;

use hp67emu::machines::hp67::{
    Hp67CardTrack, Hp67MagneticCard, Hp67MagneticTrack, TeenixHppImport,
};

use crate::ui::program_card::{ProgramCardArtwork, MOON_ROCKET_LANDER_CARD};

macro_rules! hpp {
    ($path:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
    };
}

#[derive(Debug, Clone)]
pub struct LoadedProgramCard {
    pub card: Hp67MagneticCard,
    pub card_name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ProgramLibraryEntry {
    pub pack: &'static str,
    pub reference: &'static str,
    pub title: &'static str,
    pub source_pdf: Option<&'static str>,
    pub artwork_path: Option<&'static str>,
    pub parts: &'static [&'static [u8]],
    pub artwork: ProgramCardArtwork,
}

impl ProgramLibraryEntry {
    pub fn load_card(&self) -> Result<LoadedProgramCard, String> {
        let mut card = Hp67MagneticCard::default();
        let mut bitmap_name: Option<String> = None;
        let mut track_1_seen = false;
        let mut track_2_seen = false;

        for (part_index, bytes) in self.parts.iter().enumerate() {
            let imported = TeenixHppImport::from_bytes(bytes).map_err(|error| {
                format!("{} {}: invalid .hpp: {error:?}", self.reference, self.title)
            })?;
            if imported.calculator_id != 67 {
                return Err(format!(
                    "{} {} is an HP-{} card, not HP-67",
                    self.reference, self.title, imported.calculator_id
                ));
            }
            let physical_track = self
                .physical_track_override(part_index)
                .unwrap_or(imported.card_track);

            if let Some(name) = bitmap_name.as_deref() {
                if name != imported.bitmap_name
                    && self.physical_track_override(part_index).is_none()
                {
                    return Err(format!(
                        "{} {} groups mismatched card artwork IDs '{}' and '{}'",
                        self.reference, self.title, name, imported.bitmap_name
                    ));
                }
            } else {
                bitmap_name = Some(imported.bitmap_name.clone());
            }

            let seen = match physical_track {
                Hp67CardTrack::Track1 => &mut track_1_seen,
                Hp67CardTrack::Track2 => &mut track_2_seen,
            };
            if *seen {
                return Err(format!(
                    "{} {} contains duplicate {:?}",
                    self.reference, self.title, physical_track
                ));
            }
            *seen = true;
            card.set_track(physical_track, imported.track);
        }

        if !track_1_seen && !track_2_seen {
            return Err(format!(
                "{} {} contains no magnetic tracks",
                self.reference, self.title
            ));
        }

        Ok(LoadedProgramCard {
            card,
            card_name: self.title.to_owned(),
        })
    }

    pub fn track_count(&self) -> usize {
        self.parts.len()
    }

    pub fn artwork_atlas_row(&self) -> Option<usize> {
        match self.reference {
            "SD1-01A" => Some(0),
            "SD1-02A" => Some(1),
            "SD1-03A" => Some(2),
            "SD1-04A" => Some(3),
            "SD1-05A" => Some(4),
            "SD1-06A" => Some(5),
            "SD1-07A" => Some(6),
            "SD1-08A" => Some(7),
            "SD1-09A" => Some(8),
            "SD1-10A" => Some(9),
            "SD1-11B" => Some(10),
            "SD1-12A" => Some(11),
            "SD1-13A" => Some(12),
            "SD1-14A" => Some(13),
            "SD1-15A" => Some(14),
            "GA1-01A" => Some(15),
            "GA1-02A" => Some(16),
            "GA1-03A" => Some(17),
            "GA1-04A" => Some(18),
            "GA1-05A" => Some(19),
            "GA1-06A1" => Some(20),
            "GA1-06A2" => Some(21),
            "GA1-07A" => Some(22),
            "GA1-08A" => Some(23),
            "GA1-09A" => Some(24),
            "GA1-10A" => Some(25),
            "GA1-11A" => Some(26),
            "GA1-12A" => Some(27),
            "GA1-13A" => Some(28),
            "GA1-14A" => Some(29),
            "GA1-15A" => Some(30),
            "GA1-16A" => Some(31),
            "GA1-17A" => Some(32),
            "GA1-18A" => Some(33),
            "GA1-19A" => Some(34),
            _ => None,
        }
    }

    pub fn program_listing(&self) -> Result<String, String> {
        let mut listing = String::new();

        for (part_index, bytes) in self.parts.iter().enumerate() {
            let imported = TeenixHppImport::from_bytes(bytes).map_err(|error| {
                format!("{} {}: invalid .hpp: {error:?}", self.reference, self.title)
            })?;

            if part_index > 0 {
                listing.push('\n');
            }

            match imported.header_id {
                3 | 4 => {
                    if self.parts.len() > 1 {
                        let side = self
                            .physical_track_override(part_index)
                            .unwrap_or(imported.card_track);
                        let side_number = match side {
                            Hp67CardTrack::Track1 => 1,
                            Hp67CardTrack::Track2 => 2,
                        };
                        writeln!(
                            listing,
                            "SIDE {side_number}  ·  PROGRAM HEADER {}",
                            imported.header_id
                        )
                        .expect("writing to String cannot fail");
                        writeln!(listing, "--------------------------------")
                            .expect("writing to String cannot fail");
                    }

                    let base_step = if imported.header_id == 4 { 113 } else { 1 };
                    let program = program_bytes_from_track(&imported.track)?;
                    for (offset, code) in program.into_iter().enumerate() {
                        writeln!(
                            listing,
                            "{:03}  {:02X}  {}",
                            base_step + offset,
                            code,
                            hp67_program_mnemonic(code)
                        )
                        .expect("writing to String cannot fail");
                    }
                }
                1 | 2 => {
                    writeln!(
                        listing,
                        "SIDE {}  ·  DATA CARD (HEADER {})",
                        part_index + 1,
                        imported.header_id
                    )
                    .expect("writing to String cannot fail");
                    writeln!(listing, "No user-program listing is stored on this side.")
                        .expect("writing to String cannot fail");
                }
                header => {
                    writeln!(
                        listing,
                        "SIDE {}  ·  UNKNOWN CARD HEADER {header}",
                        part_index + 1
                    )
                    .expect("writing to String cannot fail");
                }
            }
        }

        Ok(listing)
    }

    fn physical_track_override(&self, part_index: usize) -> Option<Hp67CardTrack> {
        match (self.reference, part_index) {
            // HP's Standard Pac documents SD1-12A as one physical card whose two
            // sides are independent one-pass programs. Both Teenix payloads therefore
            // carry program header 3, while the distinct A1/A2 artwork identifies the
            // physical side. Header class alone cannot select the card end here.
            ("SD1-12A", 0) => Some(Hp67CardTrack::Track1),
            ("SD1-12A", 1) => Some(Hp67CardTrack::Track2),
            _ => None,
        }
    }
}

const PROGRAM_STEPS_PER_CARD_SIDE: usize = 112;

fn program_bytes_from_track(
    track: &Hp67MagneticTrack,
) -> Result<[u8; PROGRAM_STEPS_PER_CARD_SIDE], String> {
    let words = track
        .words()
        .ok_or_else(|| "program listing requested from an unrecorded track".to_owned())?;
    let mut nibbles = [0u8; PROGRAM_STEPS_PER_CARD_SIDE * 2];
    let mut output = 0usize;

    // HP-67 program-card firmware stores one 14-nibble RAM register from each
    // pair of seven-nibble CRC records, with the later record occupying the
    // low-address half. Record 0 is the card header and record 33 is not part
    // of the 112 user-program bytes.
    for pair in 0..16 {
        for record in [2 + pair * 2, 1 + pair * 2] {
            let word = words[record];
            for nibble in 0..7 {
                nibbles[output] = ((word >> (nibble * 4)) & 0x0f) as u8;
                output += 1;
            }
        }
    }

    let mut program = [0u8; PROGRAM_STEPS_PER_CARD_SIDE];
    for (index, byte) in program.iter_mut().enumerate() {
        *byte = nibbles[index * 2] | (nibbles[index * 2 + 1] << 4);
    }
    Ok(program)
}

fn hp67_operand(nibble: u8) -> String {
    match nibble {
        0x0..=0x9 => char::from(b'0' + nibble).to_string(),
        0x0a => "A".to_owned(),
        0x0b => "B".to_owned(),
        0x0c => "C".to_owned(),
        0x0d => "D".to_owned(),
        0x0e => "E".to_owned(),
        _ => "(i)".to_owned(),
    }
}

fn hp67_program_mnemonic(code: u8) -> String {
    let fixed = match code {
        0x00 => Some("R/S"),
        0x01 => Some("1/x"),
        0x02 => Some("x^2"),
        0x03 => Some("sqrt(x)"),
        0x04 => Some("%"),
        0x05 => Some("Sigma+"),
        0x06 => Some("y^x"),
        0x07 => Some("ln"),
        0x08 => Some("e^x"),
        0x09 => Some("R->P"),
        0x0a => Some("SIN"),
        0x0b => Some("COS"),
        0x0c => Some("TAN"),
        0x0d => Some("P->R"),
        0x0e => Some("RTN"),
        0x0f => Some("RCL Sigma"),
        0x1a => Some("."),
        0x1b => Some("ENTER"),
        0x1c => Some("CHS"),
        0x1d => Some("EEX"),
        0x1e => Some("/"),
        0x1f => Some("(i) [unused]"),
        0x20 => Some("PAUSE"),
        0x21 => Some("n!"),
        0x22 => Some("MEAN"),
        0x23 => Some("SDEV"),
        0x24 => Some("%CH"),
        0x25 => Some("Sigma-"),
        0x26 => Some("ABS"),
        0x27 => Some("LOG"),
        0x28 => Some("10^x"),
        0x29 => Some("INT"),
        0x2a => Some("ASIN"),
        0x2b => Some("ACOS"),
        0x2c => Some("ATAN"),
        0x2d => Some("FRAC"),
        0x2e => Some("RND"),
        0x2f => Some("g RND [unused]"),
        0x30 => Some("x<>y"),
        0x31 => Some("RDN"),
        0x32 => Some("CLx"),
        0x33 => Some("ENG"),
        0x34 => Some("FIX"),
        0x35 => Some("PRTx"),
        0x36 => Some("SCI"),
        0x37 => Some("+"),
        0x38 => Some("-"),
        0x39 => Some("*"),
        0x3a => Some("D->R"),
        0x3b => Some("R->D"),
        0x3c => Some("H->HMS"),
        0x3d => Some("HMS->H"),
        0x3e => Some("STO (i)"),
        0x3f => Some("RCL (i)"),
        0x40 => Some("HMS+"),
        0x41 => Some("SPACE"),
        0x42 => Some("PRSTK"),
        0x43 => Some("LASTx"),
        0x44 => Some("WDATA"),
        0x45 => Some("MERGE"),
        0x46 => Some("x<>I"),
        0x47 => Some("R^"),
        0x48 => Some("PI"),
        0x49 => Some("DEG"),
        0x4a => Some("RAD"),
        0x4b => Some("GRAD"),
        0x4c => Some("P<>S"),
        0x4d => Some("CLREG"),
        0x4e => Some("PREG"),
        0x4f => Some("h PI [unused]"),
        0x50 => Some("x!=y"),
        0x51 => Some("x=y"),
        0x52 => Some("x>y"),
        0x53 => Some("x!=0"),
        0x54 => Some("x=0"),
        0x55 => Some("x>0"),
        0x56 => Some("x<0"),
        0x57 => Some("x<=y"),
        0x5c => Some("ISZ"),
        0x5d => Some("ISZ (i)"),
        0x5e => Some("DSZ"),
        0x5f => Some("DSZ (i)"),
        0x6e => Some("CF 4 [unused]"),
        0x6f => Some("DSP (i)"),
        0xff => Some("LBL (i) [unused]"),
        _ => None,
    };
    if let Some(name) = fixed {
        return name.to_owned();
    }

    match code {
        0x10..=0x19 => format!("{}", code - 0x10),
        0x58..=0x5b => format!("F? {}", code - 0x58),
        0x60..=0x69 => format!("DSP {}", code - 0x60),
        0x6a..=0x6d => format!("CF {}", code - 0x6a),
        0x70..=0x7f => format!("RCL {}", hp67_operand(code & 0x0f)),
        0x80..=0x89 => format!("STO / {}", code & 0x0f),
        0x8a..=0x8d => format!("SF {}", code - 0x8a),
        0x8e => "SF 4 [unused]".to_owned(),
        0x8f => "STO / (i)".to_owned(),
        0x90..=0x9f => format!("STO {}", hp67_operand(code & 0x0f)),
        0xa0..=0xaf => format!("STO - {}", hp67_operand(code & 0x0f)),
        0xb0..=0xbf => format!("GSB {}", hp67_operand(code & 0x0f)),
        0xc0..=0xcf => format!("STO + {}", hp67_operand(code & 0x0f)),
        0xd0..=0xdf => format!("GTO {}", hp67_operand(code & 0x0f)),
        0xe0..=0xef => format!("STO * {}", hp67_operand(code & 0x0f)),
        0xf0..=0xff => format!("LBL {}", hp67_operand(code & 0x0f)),
        _ => format!("OP {code:02X}"),
    }
}

const fn catalog_artwork(title: &'static str, reference: &'static str) -> ProgramCardArtwork {
    ProgramCardArtwork {
        title,
        reference,
        primary_labels: ["", "", "", "", ""],
        shifted_labels: ["", "", "", "", ""],
        top_marks: &[],
        show_hp_logo: true,
    }
}

pub const PROGRAM_LIBRARY: &[ProgramLibraryEntry] = &[
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "01",
        title: "Area Of Circle",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/01_AreaOfCircle.hpp")],
        artwork: catalog_artwork("Area Of Circle", "01"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "02",
        title: "Pythagorean",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/02_Pythagorean.hpp")],
        artwork: catalog_artwork("Pythagorean", "02"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "03",
        title: "Future Savings",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/03_FutureSavings.hpp")],
        artwork: catalog_artwork("Future Savings", "03"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "04",
        title: "Time To Fall",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/04_TimeToFall.hpp")],
        artwork: catalog_artwork("Time To Fall", "04"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "05",
        title: "Cumulative Total",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/05_CumulativeTotal.hpp")],
        artwork: catalog_artwork("Cumulative Total", "05"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "06",
        title: "Base Areas",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/06_BaseAreas.hpp")],
        artwork: catalog_artwork("Base Areas", "06"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "07",
        title: "Average Of 3",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/07_AverageOf3.hpp")],
        artwork: catalog_artwork("Average Of 3", "07"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "08",
        title: "Square Roots",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/08_SquareRoots.hpp")],
        artwork: catalog_artwork("Square Roots", "08"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "09",
        title: "Accountants",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/09_Accountants.hpp")],
        artwork: catalog_artwork("Accountants", "09"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "10",
        title: "Calculate e",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/10_Calculate_e.hpp")],
        artwork: catalog_artwork("Calculate e", "10"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "11",
        title: "Common Logs",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/11_CommonLogs.hpp")],
        artwork: catalog_artwork("Common Logs", "11"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "12",
        title: "Quadratic Roots",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/12_QuadraticRoots.hpp")],
        artwork: catalog_artwork("Quadratic Roots", "12"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "13",
        title: "Dice",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/13_Dice.hpp")],
        artwork: catalog_artwork("Dice", "13"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "14",
        title: "I Register",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/14_I_Register.hpp")],
        artwork: catalog_artwork("I Register", "14"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "15",
        title: "Manhattan Value",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/15_ManhattanValue.hpp")],
        artwork: catalog_artwork("Manhattan Value", "15"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "16",
        title: "Register I",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/16_Register_I.hpp")],
        artwork: catalog_artwork("Register I", "16"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "17",
        title: "I Addressing",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/17_I_Addressing.hpp")],
        artwork: catalog_artwork("I Addressing", "17"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "18",
        title: "I Average",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/18_I_Average.hpp")],
        artwork: catalog_artwork("I Average", "18"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "19",
        title: "I Random",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/19_I_Random.hpp")],
        artwork: catalog_artwork("I Random", "19"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "20",
        title: "Fibonacci",
        source_pdf: None,
        artwork_path: None,
        parts: &[
            hpp!("/programs/HP67/DemoPac1/20_Fibonacci_1.hpp"),
            hpp!("/programs/HP67/DemoPac1/20_Fibonacci_2.hpp"),
        ],
        artwork: catalog_artwork("Fibonacci", "20"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "21",
        title: "Dist Spd Time",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/21_DistSpdTime.hpp")],
        artwork: catalog_artwork("Dist Spd Time", "21"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "22",
        title: "Flags",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/22_Flags.hpp")],
        artwork: catalog_artwork("Flags", "22"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "HP67diagA",
        title: "HP-67 Diagnostic A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/HP67diagA.hpp")],
        artwork: catalog_artwork("HP-67 Diagnostic A", "HP67diagA"),
    },
    ProgramLibraryEntry {
        pack: "Demo Pac 1",
        reference: "HP67diagB",
        title: "HP-67 Diagnostic B",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/HP67diagB.hpp")],
        artwork: catalog_artwork("HP-67 Diagnostic B", "HP67diagB"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-01A",
        title: "Game of 21 (Blackjack)",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-01A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-01A_1 Game of 21 (Blackjack).hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-01A_2 Game of 21 (Blackjack).hpp"),
        ],
        artwork: catalog_artwork("Game of 21 (Blackjack)", "GA1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-02A",
        title: "Dice",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-02A.png"),
        parts: &[hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-02A_1 Dice.hpp")],
        artwork: catalog_artwork("Dice", "GA1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-03A",
        title: "Slot Machine",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-03A.png"),
        parts: &[hpp!(
            "/programs/HP67/HP-67 Games Pac 1/GA1-03A_1 Slot Machine.hpp"
        )],
        artwork: catalog_artwork("Slot Machine", "GA1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-04A",
        title: "Submarine Hunt",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-04A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-04A_1 Submarine Hunt.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-04A_2 Submarine Hunt.hpp"),
        ],
        artwork: catalog_artwork("Submarine Hunt", "GA1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-05A",
        title: "Artillery",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-05A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-05A_1 Artillery.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-05A_2 Artillery.hpp"),
        ],
        artwork: catalog_artwork("Artillery", "GA1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-06A1",
        title: "Space War 1",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-06A1.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-06A1_1 Space War 1.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-06A1_2 Space War 1.hpp"),
        ],
        artwork: catalog_artwork("Space War 1", "GA1-06A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-06A2",
        title: "Space War 2",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-06A2.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-06A2_1 Space War 2.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-06A2_2 Space War 2.hpp"),
        ],
        artwork: catalog_artwork("Space War 2", "GA1-06A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-07A",
        title: "Super Bagels",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-07A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-07A_1 Super Bagels.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-07A_2 Super Bagels.hpp"),
        ],
        artwork: catalog_artwork("Super Bagels", "GA1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-08A",
        title: "NIM",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-08A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-08A_1 NIM.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-08A_2 NIM.hpp"),
        ],
        artwork: catalog_artwork("NIM", "GA1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-09A",
        title: "Queen Board",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-09A.png"),
        parts: &[hpp!(
            "/programs/HP67/HP-67 Games Pac 1/GA1-09A_1 Queen Board.hpp"
        )],
        artwork: catalog_artwork("Queen Board", "GA1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-10A",
        title: "Hexapawn",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-10A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-10A_1 Hexapawn.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-10A_2 Hexapawn.hpp"),
        ],
        artwork: catalog_artwork("Hexapawn", "GA1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-11A",
        title: "Tic-Tac-Toe",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-11A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-11A_1 Tic-Tac-Toe.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-11A_2 Tic-Tac-Toe.hpp"),
        ],
        artwork: catalog_artwork("Tic-Tac-Toe", "GA1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-12A",
        title: "Wari",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-12A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-12A_1 Wari.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-12A_2 Wari.hpp"),
        ],
        artwork: catalog_artwork("Wari", "GA1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-13A",
        title: "Racetrack",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-13A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-13A_1 Racetrack.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-13A_2 Racetrack.hpp"),
        ],
        artwork: catalog_artwork("Racetrack", "GA1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-14A",
        title: "Teaser",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-14A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-14A_1 Teaser.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-14A_2 Teaser.hpp"),
        ],
        artwork: catalog_artwork("Teaser", "GA1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-15A",
        title: "Golf",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-15A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-15A_1 Golf.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-15A_2 Golf.hpp"),
        ],
        artwork: catalog_artwork("Golf", "GA1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-16A",
        title: "The Dealer",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-16A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-16A_1 The Dealer.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-16A_2 The Dealer.hpp"),
        ],
        artwork: catalog_artwork("The Dealer", "GA1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-17A",
        title: "Bowling Scorekeeper",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-17A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-17A_1 Bowling Scorekeeper.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-17A_2 Bowling Scorekeeper.hpp"),
        ],
        artwork: catalog_artwork("Bowling Scorekeeper", "GA1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-18A",
        title: "BioRhythms",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-18A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-18A_1 BioRhythms.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-18A_2 BioRhythms.hpp"),
        ],
        artwork: catalog_artwork("BioRhythms", "GA1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-19A",
        title: "Timer",
        source_pdf: Some("hp6797-pac-games-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/GA1-19A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-19A_1 Timer.hpp"),
            hpp!("/programs/HP67/HP-67 Games Pac 1/GA1-19A_2 Timer.hpp"),
        ],
        artwork: catalog_artwork("Timer", "GA1-19A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-01A",
        title: "Moving Average",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-01A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-01A_1 Moving Average.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-01A_2 Moving Average.hpp"),
        ],
        artwork: catalog_artwork("Moving Average", "SD1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-02A",
        title: "Tabulator",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-02A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-02A_1 Tabulator.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-02A_2 Tabulator.hpp"),
        ],
        artwork: catalog_artwork("Tabulator", "SD1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-03A",
        title: "Curve Fitting",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-03A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-03A_1 Curve Fitting.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-03A_2 Curve Fitting.hpp"),
        ],
        artwork: catalog_artwork("Curve Fitting", "SD1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-04A",
        title: "Calendar Functions",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-04A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-04A_1 Calendar Functions.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-04A_2 Calendar Functions.hpp"),
        ],
        artwork: catalog_artwork("Calendar Functions", "SD1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-05A",
        title: "Annuities and Compound Amounts",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-05A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-05A_1 Annuities and Compound Amounts.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-05A_2 Annuities and Compound Amounts.hpp"),
        ],
        artwork: catalog_artwork("Annuities and Compound Amounts", "SD1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-06A",
        title: "Follow Me",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-06A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-06A_1 Follow Me.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-06A_2 Follow Me.hpp"),
        ],
        artwork: catalog_artwork("Follow Me", "SD1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-07A",
        title: "Triangle Solutions",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-07A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-07A_1 Triangle Solutions.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-07A_2 Triangle Solutions.hpp"),
        ],
        artwork: catalog_artwork("Triangle Solutions", "SD1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-08A",
        title: "Vector Operations",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-08A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-08A_1 Vector Operations.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-08A_2 Vector Operations.hpp"),
        ],
        artwork: catalog_artwork("Vector Operations", "SD1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-09A",
        title: "Polynomial Evaluation",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-09A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-09A_1 Polynomial Evaluation.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-09A_2 Polynomial Evaluation.hpp"),
        ],
        artwork: catalog_artwork("Polynomial Evaluation", "SD1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-10A",
        title: "Matrix Operations",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-10A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-10A_1 Matrix Operations.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-10A_2 Matrix Operations.hpp"),
        ],
        artwork: catalog_artwork("Matrix Operations", "SD1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-11B",
        title: "Calculus and Roots of f(x)",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-11B.png"),
        parts: &[hpp!(
            "/programs/HP67/HP-67 Standard Pac/SD1-11B_1 Calculus and Roots of f(x).hpp"
        )],
        artwork: catalog_artwork("Calculus and Roots of f(x)", "SD1-11B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-12A",
        title: "English-SI Conversions",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-12A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-12A_1 English-SI Conversions.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-12A_2 English-SI Conversions.hpp"),
        ],
        artwork: catalog_artwork("English-SI Conversions", "SD1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-13A",
        title: "Arithmetic Teacher",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-13A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-13A_1 Arithmetic Teacher.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-13A_2 Arithmetic Teacher.hpp"),
        ],
        artwork: catalog_artwork("Arithmetic Teacher", "SD1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-14A",
        title: "Moon Rocket Lander",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-14A.png"),
        parts: &[hpp!(
            "/programs/HP67/HP-67 Standard Pac/SD1-14A_1 Moon Rocket Lander.hpp"
        )],
        artwork: MOON_ROCKET_LANDER_CARD,
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-15A",
        title: "Diagnostic Program",
        source_pdf: Some("hp67-pac-standard-en.pdf"),
        artwork_path: Some("programs/HP67/_artwork/SD1-15A.png"),
        parts: &[
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-15A_1 Diagnostic Program.hpp"),
            hpp!("/programs/HP67/HP-67 Standard Pac/SD1-15A_2 Diagnostic Program.hpp"),
        ],
        artwork: catalog_artwork("Diagnostic Program", "SD1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Diagnostic Cards",
        reference: "SD-15C",
        title: "Diagnostic Program (SD-15C)",
        source_pdf: None,
        artwork_path: None,
        parts: &[
            hpp!("/programs/HP67/HP-67 Diagnostic Cards/SD-15C-Diagnostic-Program_1.hpp"),
            hpp!("/programs/HP67/HP-67 Diagnostic Cards/SD-15C-Diagnostic-Program_2.hpp"),
        ],
        artwork: catalog_artwork("Diagnostic Program", "SD-15C"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-01",
        title: "Flow GSB/GTO/RTN — expect 7.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-01_Flow-GSB-GTO-RTN_1.hpp"
        )],
        artwork: catalog_artwork("Flow GSB/GTO/RTN", "CD-01"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-02",
        title: "Flags SF/CF/F? — expect 6.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-02_Flags-SF-CF-Test_1.hpp"
        )],
        artwork: catalog_artwork("Flags SF/CF/F?", "CD-02"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-03",
        title: "Conditionals — expect 7.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-03_Conditionals_1.hpp"
        )],
        artwork: catalog_artwork("Conditionals", "CD-03"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-04",
        title: "Indirect STO/RCL — expect 42.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-04_Indirect-STO-RCL_1.hpp"
        )],
        artwork: catalog_artwork("Indirect STO/RCL", "CD-04"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-05",
        title: "Nested GSB — expect 6.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-05_Nested-GSB_1.hpp"
        )],
        artwork: catalog_artwork("Nested GSB", "CD-05"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-06",
        title: "Second-half label search — expect 67.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[
            hpp!("/programs/HP67/Custom Diagnostic Pacs/CD-06_Second-Half-Label-Search_1.hpp"),
            hpp!("/programs/HP67/Custom Diagnostic Pacs/CD-06_Second-Half-Label-Search_2.hpp"),
        ],
        artwork: catalog_artwork("Second-half label search", "CD-06"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-07",
        title: "ISZ loop — expect 3.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-07_ISZ-Loop_1.hpp"
        )],
        artwork: catalog_artwork("ISZ loop", "CD-07"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-08",
        title: "DSZ loop — expect 3.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-08_DSZ-Loop_1.hpp"
        )],
        artwork: catalog_artwork("DSZ loop", "CD-08"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-09",
        title: "Long DSZ burn-in — expect 2000.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-09_Long-DSZ-Burn-In_1.hpp"
        )],
        artwork: catalog_artwork("Long DSZ burn-in", "CD-09"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-10",
        title: "Nested GSB burn-in — expect 500.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-10_Nested-GSB-Burn-In_1.hpp"
        )],
        artwork: catalog_artwork("Nested GSB burn-in", "CD-10"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-11",
        title: "Function identity burn-in — expect 1.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/Custom Diagnostic Pacs/CD-11_Function-Identity-Burn-In_1.hpp"
        )],
        artwork: catalog_artwork("Function identity burn-in", "CD-11"),
    },
    ProgramLibraryEntry {
        pack: "Custom Diagnostic Pacs",
        reference: "CD-12",
        title: "Cross-half GSB burn-in — expect 500.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[
            hpp!("/programs/HP67/Custom Diagnostic Pacs/CD-12_Cross-Half-GSB-Burn-In_1.hpp"),
            hpp!("/programs/HP67/Custom Diagnostic Pacs/CD-12_Cross-Half-GSB-Burn-In_2.hpp"),
        ],
        artwork: catalog_artwork("Cross-half GSB burn-in", "CD-12"),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pdf_backed_card_has_a_unique_embedded_artwork_row() {
        let mut rows = PROGRAM_LIBRARY
            .iter()
            .filter(|entry| entry.source_pdf.is_some())
            .map(|entry| {
                entry.artwork_atlas_row().unwrap_or_else(|| {
                    panic!(
                        "{} {} lacks embedded PDF artwork",
                        entry.reference, entry.title
                    )
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 35);
        rows.sort_unstable();
        rows.dedup();
        assert_eq!(rows, (0..35).collect::<Vec<_>>());
    }

    #[test]
    fn moon_rocket_listing_decodes_program_bytes_from_card_records() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD1-14A")
            .expect("Moon Rocket Lander must be in the catalog");
        let listing = entry
            .program_listing()
            .expect("Moon Rocket listing must decode");
        assert!(listing.contains("001  FA  LBL A"));
        assert!(listing.contains("002  15  5"));
    }

    #[test]
    fn continuation_card_listing_uses_second_half_step_numbers() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "20")
            .expect("Fibonacci must be in the catalog");
        let listing = entry
            .program_listing()
            .expect("Fibonacci listing must decode");
        assert!(listing.contains("SIDE 2"));
        assert!(listing.lines().any(|line| line.starts_with("113  ")));
    }

    #[test]
    fn catalog_contains_all_checked_in_hpp_programs() {
        assert_eq!(PROGRAM_LIBRARY.len(), 72);
        assert_eq!(
            PROGRAM_LIBRARY
                .iter()
                .map(ProgramLibraryEntry::track_count)
                .sum::<usize>(),
            106
        );
    }

    #[test]
    fn english_si_card_keeps_two_independent_header_three_sides() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD1-12A")
            .unwrap();
        let loaded = entry.load_card().unwrap();
        assert_eq!(
            loaded
                .card
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(3)
        );
        assert_eq!(
            loaded
                .card
                .track(Hp67CardTrack::Track2)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(3)
        );
    }

    #[test]
    fn custom_diagnostic_pack_contains_twelve_known_result_cards() {
        let entries = PROGRAM_LIBRARY
            .iter()
            .filter(|entry| entry.pack == "Custom Diagnostic Pacs")
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 12);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.reference)
                .collect::<Vec<_>>(),
            [
                "CD-01", "CD-02", "CD-03", "CD-04", "CD-05", "CD-06", "CD-07", "CD-08",
                "CD-09", "CD-10", "CD-11", "CD-12",
            ]
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.track_count())
                .sum::<usize>(),
            14
        );
    }

    #[test]
    fn custom_diagnostic_native_cards_match_their_library_hpp_tracks() {
        let cases: [(&str, &[u8]); 12] = [
            (
                "CD-01",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-01_Flow-GSB-GTO-RTN.hp67card"
                )),
            ),
            (
                "CD-02",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-02_Flags-SF-CF-Test.hp67card"
                )),
            ),
            (
                "CD-03",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-03_Conditionals.hp67card"
                )),
            ),
            (
                "CD-04",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-04_Indirect-STO-RCL.hp67card"
                )),
            ),
            (
                "CD-05",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-05_Nested-GSB.hp67card"
                )),
            ),
            (
                "CD-06",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-06_Second-Half-Label-Search.hp67card"
                )),
            ),
            (
                "CD-07",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-07_ISZ-Loop.hp67card"
                )),
            ),
            (
                "CD-08",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-08_DSZ-Loop.hp67card"
                )),
            ),
            (
                "CD-09",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-09_Long-DSZ-Burn-In.hp67card"
                )),
            ),
            (
                "CD-10",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-10_Nested-GSB-Burn-In.hp67card"
                )),
            ),
            (
                "CD-11",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-11_Function-Identity-Burn-In.hp67card"
                )),
            ),
            (
                "CD-12",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/programs/HP67/Custom Diagnostic Pacs/CD-12_Cross-Half-GSB-Burn-In.hp67card"
                )),
            ),
        ];

        for (reference, native_bytes) in cases {
            let native = Hp67MagneticCard::from_hp67card_bytes(native_bytes)
                .unwrap_or_else(|error| panic!("{reference}: {error:?}"));
            let entry = PROGRAM_LIBRARY
                .iter()
                .find(|entry| entry.reference == reference)
                .expect("custom diagnostic must be in the catalog");
            let imported = entry.load_card().unwrap().card;
            assert_eq!(native, imported, "{reference} native/HPP media mismatch");
        }
    }

    #[test]
    fn standard_diagnostic_native_card_matches_source_backed_hpp_pair() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD1-15A")
            .expect("Standard Pac diagnostic must be in the catalog");
        let imported = entry.load_card().unwrap().card;
        let native = Hp67MagneticCard::from_hp67card_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/HP-67 Standard Pac/SD1-15A-Diagnostic-Program.hp67card"
        )))
        .expect("checked-in native Standard diagnostic must decode");
        assert_eq!(native, imported);
    }

    #[test]
    fn exact_sd15c_native_card_matches_embedded_hpp_wrappers() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD-15C")
            .expect("SD-15C must be in the catalog");
        let imported = entry.load_card().unwrap().card;
        let native = Hp67MagneticCard::from_hp67card_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/HP-67 Diagnostic Cards/SD-15C-Diagnostic-Program.hp67card"
        )))
        .expect("exact SD-15C native fixture must decode");
        assert_eq!(native, imported);
        assert_eq!(
            native
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(3)
        );
        assert_eq!(
            native
                .track(Hp67CardTrack::Track2)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(4)
        );

        let listing = entry.program_listing().unwrap();
        assert!(listing.contains("035  FA  LBL A"));
        assert!(listing.contains("113  F8  LBL 8"));
    }

    #[test]
    fn second_half_custom_diagnostic_reaches_program_steps_113_through_224() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "CD-06")
            .expect("CD-06 must be in the catalog");
        let loaded = entry.load_card().unwrap();
        assert_eq!(
            loaded
                .card
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(3)
        );
        assert_eq!(
            loaded
                .card
                .track(Hp67CardTrack::Track2)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(4)
        );
        let listing = entry.program_listing().unwrap();
        assert!(listing.contains("001  FA  LBL A"));
        assert!(listing.contains("113  FE  LBL E"));
    }

    #[test]
    fn every_catalog_entry_decodes_to_one_physical_card() {
        for entry in PROGRAM_LIBRARY {
            let loaded = entry.load_card().unwrap_or_else(|error| panic!("{error}"));
            assert!(
                loaded.card.track(Hp67CardTrack::Track1).is_recorded()
                    || loaded.card.track(Hp67CardTrack::Track2).is_recorded(),
                "{} {} decoded without a recorded track",
                entry.reference,
                entry.title
            );
        }
    }
}
