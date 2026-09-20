use hp67emu::machines::hp67::{Hp67CardTrack, Hp67MagneticCard, TeenixHppImport};

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
                if name != imported.bitmap_name && self.physical_track_override(part_index).is_none()
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
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_all_checked_in_hpp_programs() {
        assert_eq!(PROGRAM_LIBRARY.len(), 59);
        assert_eq!(
            PROGRAM_LIBRARY
                .iter()
                .map(ProgramLibraryEntry::track_count)
                .sum::<usize>(),
            90
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
