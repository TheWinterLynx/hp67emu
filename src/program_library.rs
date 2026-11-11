use std::fmt::Write;

use hp67emu::machines::hp67::{
    Hp67CardTrack, Hp67MagneticCard, Hp67MagneticTrack, TeenixHppImport,
};

use crate::ui::program_card::ProgramCardArtwork;

include!("../assets/embedded-card-artwork.rs");

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
        if self.parts.len() == 1 && self.parts[0].starts_with(b"HP67CARD") {
            let card = Hp67MagneticCard::from_hp67card_bytes(self.parts[0]).map_err(|error| {
                format!(
                    "{} {}: invalid native .hp67card: {error:?}",
                    self.reference, self.title
                )
            })?;
            return Ok(LoadedProgramCard {
                card,
                card_name: self.title.to_owned(),
            });
        }
        if self
            .parts
            .iter()
            .any(|bytes| bytes.starts_with(b"HP67CARD"))
        {
            return Err(format!(
                "{} {} mixes native .hp67card media with compatibility payloads",
                self.reference, self.title
            ));
        }

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
        if self.parts.len() == 1 && self.parts[0].starts_with(b"HP67CARD") {
            let bytes = self.parts[0];
            return usize::from(bytes.get(9).is_some_and(|flags| flags & 0x01 != 0))
                + usize::from(bytes.get(10).is_some_and(|flags| flags & 0x01 != 0));
        }
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
            "SD-15C" => Some(14),
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
            "BD1-01B" => Some(35),
            "BD1-02B" => Some(36),
            "BD1-03A" => Some(37),
            "BD1-04A" => Some(38),
            "BD1-05A" => Some(39),
            "BD1-06A" => Some(40),
            "BD1-07A" => Some(41),
            "BD1-08A" => Some(42),
            "BD1-09A" => Some(43),
            "BD1-10A" => Some(44),
            "BD1-11A" => Some(45),
            "BD1-12A" => Some(46),
            "BD1-13A" => Some(47),
            "BD1-14A" => Some(48),
            "BD1-15A" => Some(49),
            "BD1-16A" => Some(50),
            "BD1-17A" => Some(51),
            "BD1-18A" => Some(52),
            "BD1-19A" => Some(53),
            "BD1-20A" => Some(54),
            "BD1-21A" => Some(55),
            "BD1-22A" => Some(56),
            "CE1-01A" => Some(57),
            "CE1-02A1" => Some(58),
            "CE1-02B2" => Some(59),
            "CE1-03A" => Some(60),
            "CE1-04B" => Some(61),
            "CE1-05A" => Some(62),
            "CE1-06A" => Some(63),
            "CE1-07A" => Some(64),
            "CE1-08A" => Some(65),
            "CE1-09A" => Some(66),
            "CE1-10A" => Some(67),
            "CE1-11A" => Some(68),
            "CE1-12A" => Some(69),
            "CE1-13A" => Some(70),
            "CE1-14A" => Some(71),
            "CE1-15A" => Some(72),
            "CE1-16A" => Some(73),
            "CE1-17B" => Some(74),
            "CE1-18A" => Some(75),
            "EE1-01A1" => Some(76),
            "EE1-01B2" => Some(77),
            "EE1-02A" => Some(78),
            "EE1-03A" => Some(79),
            "EE1-04A" => Some(80),
            "EE1-05A" => Some(81),
            "EE1-06A" => Some(82),
            "EE1-07B" => Some(83),
            "EE1-08A" => Some(84),
            "EE1-09A1" => Some(85),
            "EE1-09A2" => Some(86),
            "EE1-10A" => Some(87),
            "EE1-11A" => Some(88),
            "EE1-12A" => Some(89),
            "EE1-13A" => Some(90),
            "EE1-14A" => Some(91),
            "EE1-15A" => Some(92),
            "EE1-16A" => Some(93),
            "EE1-17A" => Some(94),
            "EE1-18A" => Some(95),
            "MA1-01A" => Some(96),
            "MA1-02A" => Some(97),
            "MA1-03A" => Some(98),
            "MA1-04A" => Some(99),
            "MA1-05A" => Some(100),
            "MA1-06A" => Some(101),
            "MA1-07A1" => Some(102),
            "MA1-07A2" => Some(103),
            "MA1-08A" => Some(104),
            "MA1-09A" => Some(105),
            "MA1-10A" => Some(106),
            "MA1-11A" => Some(107),
            "MA1-12A" => Some(108),
            "MA1-13A" => Some(109),
            "MA1-14A" => Some(110),
            "MA1-15A" => Some(111),
            "MA1-16A" => Some(112),
            "MA1-17A" => Some(113),
            "MA1-18A" => Some(114),
            "MA1-19A" => Some(115),
            "ME1-01A1" => Some(116),
            "ME1-02A1" => Some(117),
            "ME1-02A2" => Some(118),
            "ME1-03A" => Some(119),
            "ME1-04A" => Some(120),
            "ME1-05A" => Some(121),
            "ME1-06A" => Some(122),
            "ME1-07A" => Some(123),
            "ME1-08A" => Some(124),
            "ME1-09A" => Some(125),
            "ME1-10A1" => Some(126),
            "ME1-10A2" => Some(127),
            "ME1-11A" => Some(128),
            "ME1-12A" => Some(129),
            "ME1-13A" => Some(130),
            "ME1-14A" => Some(131),
            "ME1-15A" => Some(132),
            "ME1-16A" => Some(133),
            "ME1-17A" => Some(134),
            "ME1-18A" => Some(135),
            "ME1-19B" => Some(136),
            "ME1-20A" => Some(137),
            "ME1-21A" => Some(138),
            "ME1-22A" => Some(139),
            "ME1-23A1" => Some(140),
            "ME1-23A2" => Some(141),
            "ST1-01A" => Some(142),
            "ST1-02A" => Some(143),
            "ST1-03A" => Some(144),
            "ST1-04A" => Some(145),
            "ST1-05A" => Some(146),
            "ST1-06A" => Some(147),
            "ST1-07A" => Some(148),
            "ST1-08A1" => Some(149),
            "ST1-08A2" => Some(150),
            "ST1-09A1" => Some(151),
            "ST1-09A2" => Some(152),
            "ST1-10A" => Some(153),
            "ST1-11A" => Some(154),
            "ST1-12A" => Some(155),
            "ST1-13A" => Some(156),
            "ST1-14A1" => Some(157),
            "ST1-14A2" => Some(158),
            "ST1-15A" => Some(159),
            "ST1-16A" => Some(160),
            "ST1-17A" => Some(161),
            "ST1-18A" => Some(162),
            "ST1-19A" => Some(163),
            "ST1-20A" => Some(164),
            "ST1-21A" => Some(165),
            "CL1-01-ee5c4dc6" => Some(166),
            "CL1-02-ff6f6413" => Some(167),
            "CL1-03-577a1315" => Some(168),
            "CL1-04-d8c0d74e" => Some(169),
            "CL1-05-4d38456e" => Some(170),
            "CL1-06-b8cb5a4f" => Some(171),
            "CL1-07-ed7f2e70" => Some(172),
            "CL1-08-c75972b4" => Some(173),
            "CL1-09-afcecdc8" => Some(174),
            "CL1-10-b5d7a909" => Some(175),
            "CL1-11-4db0ca8a" => Some(176),
            "CL1-12-33ae4423" => Some(177),
            "CL1-13-4cccf7c3" => Some(178),
            "CL1-14-cc726078" => Some(179),
            "CL1-15-4befaa17" => Some(180),
            "CL1-16-7d440be8" => Some(181),
            "CL1-17-7ea1b500" => Some(182),
            "CL1-18-ab9c2274" => Some(183),
            "CL1-19-a3413ecd" => Some(184),
            "NAV-01-11cf36c0" => Some(185),
            "NAV-02-182573cd" => Some(186),
            "NAV-03-e30993d3" => Some(187),
            "NAV-04-3bcb6e42" => Some(188),
            "NAV-05-6d0d4ae7" => Some(189),
            "NAV-05-D-488f90d2" => Some(190),
            "NAV-05-D-af8a9ee4" => Some(191),
            "NAV-06-12033ab1" => Some(192),
            "NAV-07-751f9bb7" => Some(193),
            "NAV-08-3dc76442" => Some(194),
            "NAV-08-0db449e9" => Some(195),
            "NAV-08-6b06eea3" => Some(196),
            "NAV-08-79aa6f37" => Some(197),
            "NAV-08-8ea27fd1" => Some(198),
            "NAV-08-b6ccb381" => Some(199),
            "NAV-08-2d858830" => Some(200),
            "NAV-09-7741f37b" => Some(201),
            "NAV-10-8906d17d" => Some(202),
            "NAV-11-b47eee2c" => Some(203),
            "NAV-12-b10aa27b" => Some(204),
            "NAV-13-61f3a634" => Some(205),
            "NAV-14-ae9cf5f6" => Some(206),
            "SU1-01-7620b7c6" => Some(207),
            "SU1-02-a1882dbf" => Some(208),
            "SU1-03-efd5b55b" => Some(209),
            "SU1-04-4501fc83" => Some(210),
            "SU1-05-72451190" => Some(211),
            "SU1-06-ac0565ca" => Some(212),
            "SU1-07-7d87d22c" => Some(213),
            "SU1-08-dbd9ff14" => Some(214),
            "SU1-09-08b9e847" => Some(215),
            "SU1-10-5685c1d9" => Some(216),
            "SU1-11-b38e2503" => Some(217),
            "SU1-12-92762b3c" => Some(218),
            "SU1-13-9a26723d" => Some(219),
            "SU1-14-2115b409" => Some(220),
            "SU1-15-784a11db" => Some(221),
            "SU1-15-2e25d058" => Some(222),
            "SU1-16-8c5158b3" => Some(223),
            "SU1-17-D-a4848e5c" => Some(224),
            "SU1-17-50de669b" => Some(225),
            "SU1-17-63bb3fe9" => Some(226),
            "SU1-17-D-965271bd" => Some(227),
            "SU1-17-D-60f4dfdd" => Some(228),
            "SU1-17-D-4f3a3be6" => Some(229),
            "SU1-17-D-fcc0ec19" => Some(230),
            "SU1-17-D-806db91f" => Some(231),
            "SU1-17-D-bfece0c8" => Some(232),
            "SU1-17-D-8f05812f" => Some(233),
            "SU1-17-D-4155b930" => Some(234),
            "SU1-17-D-34fe1da2" => Some(235),
            "SU1-17-D-b205f584" => Some(236),
            "SU1-17-D-920a1479" => Some(237),
            "SU1-17-D-7d2e871a" => Some(238),
            "SU1-17-D-a7df7a8f" => Some(239),
            "SU1-17-D-74847e7e" => Some(240),
            "SU1-17-D-49dcc1db" => Some(241),
            "SU1-17-D-b1c20e00" => Some(242),
            "SU1-17-D-f7c5fc6d" => Some(243),
            "SU1-17-D-8ab84ec6" => Some(244),
            "SU1-17-D-330f8da6" => Some(245),
            "SU1-17-D-ba23b118" => Some(246),
            "SU1-17-D-26d7e2aa" => Some(247),
            "SU1-17-D-c09c7b87" => Some(248),
            "SU1-17-D-dfe2927e" => Some(249),
            "SU1-17-D-c2b24f1a" => Some(250),
            "SU1-17-D-4be35b97" => Some(251),
            "SU1-17-D-0451902b" => Some(252),
            "SU1-17-D-fa4f5447" => Some(253),
            "SU1-17-D-35248266" => Some(254),
            "SU1-17-D-a823f8d4" => Some(255),
            "SU1-17-D-25e703dd" => Some(256),
            "SU1-17-D-578f6c07" => Some(257),
            "SU1-17-D-fd5fc2f8" => Some(258),
            "SU1-17-D-87cfc74f" => Some(259),
            "SU1-17-D-dc620c78" => Some(260),
            "SU1-17-D-999aa537" => Some(261),
            "SU1-17-D-5fab8485" => Some(262),
            "SU1-17-D-482ed621" => Some(263),
            "SU1-17-D-ee3280c1" => Some(264),
            "SU1-17-D-36ede5f9" => Some(265),
            "SU1-17-D-dd69b774" => Some(266),
            "SU1-17-D-b045f1cd" => Some(267),
            "SU1-17-D-e15eb09c" => Some(268),
            "SU1-17-D-45ef7ca0" => Some(269),
            "SU1-17-D-0b9d7685" => Some(270),
            "SU1-17-D-25e4c936" => Some(271),
            "SU1-17-D-55a528d2" => Some(272),
            "SU1-17-D-75cce508" => Some(273),
            "SU1-17-D-4457215b" => Some(274),
            "SU1-17-D-7d9e8ede" => Some(275),
            "SU1-17-D-f07f3480" => Some(276),
            "SU1-17-D-c5bb8bea" => Some(277),
            "SU1-17-D-90feaf64" => Some(278),
            "SU1-17-D-c697f9d1" => Some(279),
            "SU1-17-D-11563bbd" => Some(280),
            "SU1-17-D-2a7eeb4c" => Some(281),
            "SU1-17-D-1ffbcefc" => Some(282),
            "SU1-17-D-e932489a" => Some(283),
            "SU1-17-D-0b8ad8d5" => Some(284),
            "SU1-17-D-7c8fc840" => Some(285),
            "SU1-17-D-d9b30417" => Some(286),
            "SU1-17-D-e6c85063" => Some(287),
            "SU1-17-D-50da60f9" => Some(288),
            "SU1-17-D-ffbd2ccb" => Some(289),
            "SU1-17-D-d72aac40" => Some(290),
            "SU1-17-D-6188761e" => Some(291),
            "SU1-17-D-7a3c9074" => Some(292),
            "SU1-18-D-30e57121" => Some(293),
            "SU1-18-40f333d8" => Some(294),
            "SU1-18-1d7b2a6a" => Some(295),
            "SU1-18-D-734575da" => Some(296),
            "SU1-18-D-fc36e803" => Some(297),
            "SU1-18-D-f56a60a4" => Some(298),
            "SU1-18-D-f12f6946" => Some(299),
            "SU1-18-D-05308988" => Some(300),
            "SU1-18-D-1cf9269a" => Some(301),
            "SU1-18-D-0e899821" => Some(302),
            "SU1-18-D-d881f713" => Some(303),
            "SU1-18-D-c869a64f" => Some(304),
            "SU1-18-D-d7d659bf" => Some(305),
            "SU1-18-D-82a51873" => Some(306),
            "SU1-18-D-a436f02b" => Some(307),
            "SU1-18-D-047f92cf" => Some(308),
            "SU1-18-D-cdbe8da9" => Some(309),
            "SU1-18-D-1f2efb54" => Some(310),
            "SU1-18-D-cd62b820" => Some(311),
            "SU1-18-D-12d9097b" => Some(312),
            "SU1-18-D-1f418de0" => Some(313),
            "SU1-18-D-afb9a1d5" => Some(314),
            "SU1-18-D-8a7bc9e3" => Some(315),
            "SU1-18-D-bff2bf90" => Some(316),
            "SU1-18-D-faa6af3d" => Some(317),
            "SU1-18-D-2c492798" => Some(318),
            "SU1-18-D-5fe3925b" => Some(319),
            "SU1-18-D-4e5aae3e" => Some(320),
            "SU1-18-D-f4855de5" => Some(321),
            "SU1-18-D-dec10f2a" => Some(322),
            "SU1-18-D-8895a531" => Some(323),
            "SU1-18-D-d97a23b0" => Some(324),
            "SU1-18-D-9cbae632" => Some(325),
            "SU1-18-D-594b90a3" => Some(326),
            "SU1-18-D-4fcf10e8" => Some(327),
            "SU1-18-D-59fa689d" => Some(328),
            "SU1-18-D-bbf05970" => Some(329),
            "SU1-18-D-9bf5465f" => Some(330),
            "SU1-18-D-739f5983" => Some(331),
            "SU1-18-D-afce7ace" => Some(332),
            "SU1-18-D-eefb46c0" => Some(333),
            "SU1-18-D-e63a1cc1" => Some(334),
            "SU1-18-D-eb4cb1ea" => Some(335),
            "SU1-18-D-9318ff8f" => Some(336),
            "SU1-18-D-ec6a03a4" => Some(337),
            "SU1-18-D-c17bcd0b" => Some(338),
            "SU1-18-D-719a99d9" => Some(339),
            "SU1-18-D-497a4397" => Some(340),
            "SU1-18-D-cdcd1f3a" => Some(341),
            "SU1-18-D-e650d42f" => Some(342),
            "SU1-18-D-3cce5d22" => Some(343),
            "SU1-18-D-43ef81bd" => Some(344),
            "SU1-19-D-b75b077e" => Some(345),
            "SU1-19-02c58ff9" => Some(346),
            "SU1-19-D-c4b03467" => Some(347),
            "SU1-19-4ee33dbf" => Some(348),
            _ => None,
        }
    }

    pub fn program_listing(&self) -> Result<String, String> {
        if self.parts.len() == 1 && self.parts[0].starts_with(b"HP67CARD") {
            let card = Hp67MagneticCard::from_hp67card_bytes(self.parts[0]).map_err(|error| {
                format!(
                    "{} {}: invalid native .hp67card: {error:?}",
                    self.reference, self.title
                )
            })?;
            let mut listing = String::new();
            let side_count = self.track_count();
            let mut emitted = 0usize;
            for (track_id, side_number) in [
                (Hp67CardTrack::Track1, 1usize),
                (Hp67CardTrack::Track2, 2usize),
            ] {
                let track = card.track(track_id);
                if !track.is_recorded() {
                    continue;
                }
                if emitted > 0 {
                    listing.push('\n');
                }
                let header_id = track
                    .word(0)
                    .map(|word| ((word >> 24) & 0x0f) as u8)
                    .unwrap_or(0);
                append_program_track_listing(
                    &mut listing,
                    track,
                    header_id,
                    side_number,
                    side_count > 1,
                )?;
                emitted += 1;
            }
            return Ok(listing);
        }

        let mut listing = String::new();

        for (part_index, bytes) in self.parts.iter().enumerate() {
            let imported = TeenixHppImport::from_bytes(bytes).map_err(|error| {
                format!("{} {}: invalid .hpp: {error:?}", self.reference, self.title)
            })?;

            if part_index > 0 {
                listing.push('\n');
            }

            let physical_track = self
                .physical_track_override(part_index)
                .unwrap_or(imported.card_track);
            let side_number = match physical_track {
                Hp67CardTrack::Track1 => 1,
                Hp67CardTrack::Track2 => 2,
            };
            append_program_track_listing(
                &mut listing,
                &imported.track,
                imported.header_id,
                side_number,
                self.parts.len() > 1,
            )?;
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

fn append_program_track_listing(
    listing: &mut String,
    track: &Hp67MagneticTrack,
    header_id: u8,
    side_number: usize,
    show_side_header: bool,
) -> Result<(), String> {
    match header_id {
        3 | 4 => {
            if show_side_header {
                writeln!(listing, "SIDE {side_number}  ·  PROGRAM HEADER {header_id}")
                    .expect("writing to String cannot fail");
                writeln!(listing, "--------------------------------")
                    .expect("writing to String cannot fail");
            }

            let base_step = if header_id == 4 { 113 } else { 1 };
            let program = program_bytes_from_track(track)?;
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
                "SIDE {side_number}  ·  DATA CARD (HEADER {header_id})"
            )
            .expect("writing to String cannot fail");
            writeln!(listing, "No user-program listing is stored on this side.")
                .expect("writing to String cannot fail");
        }
        header => {
            writeln!(
                listing,
                "SIDE {side_number}  ·  UNKNOWN CARD HEADER {header}"
            )
            .expect("writing to String cannot fail");
        }
    }

    Ok(())
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

/// Stable presentation indices: catalog/media identities never move.
pub fn program_library_display_order() -> &'static [usize] {
    static ORDER: std::sync::OnceLock<Vec<usize>> = std::sync::OnceLock::new();
    ORDER.get_or_init(|| {
        let mut order: Vec<_> = (0..PROGRAM_LIBRARY.len()).collect();
        order.sort_by_cached_key(|&index| {
            let entry = &PROGRAM_LIBRARY[index];
            (
                entry.pack.to_lowercase(),
                entry.title.to_lowercase(),
                entry.reference,
                index,
            )
        });
        order
    })
}

pub const PROGRAM_LIBRARY: &[ProgramLibraryEntry] = &[
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "01",
        title: "Area Of Circle",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/01_AreaOfCircle.hpp")],
        artwork: catalog_artwork("Area Of Circle", "01"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "02",
        title: "Pythagorean",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/02_Pythagorean.hpp")],
        artwork: catalog_artwork("Pythagorean", "02"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "03",
        title: "Future Savings",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/03_FutureSavings.hpp")],
        artwork: catalog_artwork("Future Savings", "03"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "04",
        title: "Time To Fall",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/04_TimeToFall.hpp")],
        artwork: catalog_artwork("Time To Fall", "04"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "05",
        title: "Cumulative Total",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/05_CumulativeTotal.hpp")],
        artwork: catalog_artwork("Cumulative Total", "05"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "06",
        title: "Base Areas",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/06_BaseAreas.hpp")],
        artwork: catalog_artwork("Base Areas", "06"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "07",
        title: "Average Of 3",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/07_AverageOf3.hpp")],
        artwork: catalog_artwork("Average Of 3", "07"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "08",
        title: "Square Roots",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/08_SquareRoots.hpp")],
        artwork: catalog_artwork("Square Roots", "08"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "09",
        title: "Accountants",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/09_Accountants.hpp")],
        artwork: catalog_artwork("Accountants", "09"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "10",
        title: "Calculate e",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/10_Calculate_e.hpp")],
        artwork: catalog_artwork("Calculate e", "10"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "11",
        title: "Common Logs",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/11_CommonLogs.hpp")],
        artwork: catalog_artwork("Common Logs", "11"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "12",
        title: "Quadratic Roots",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/12_QuadraticRoots.hpp")],
        artwork: catalog_artwork("Quadratic Roots", "12"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "13",
        title: "Dice",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/13_Dice.hpp")],
        artwork: catalog_artwork("Dice", "13"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "14",
        title: "I Register",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/14_I_Register.hpp")],
        artwork: catalog_artwork("I Register", "14"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "15",
        title: "Manhattan Value",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/15_ManhattanValue.hpp")],
        artwork: catalog_artwork("Manhattan Value", "15"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "16",
        title: "Register I",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/16_Register_I.hpp")],
        artwork: catalog_artwork("Register I", "16"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "17",
        title: "I Addressing",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/17_I_Addressing.hpp")],
        artwork: catalog_artwork("I Addressing", "17"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "18",
        title: "I Average",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/18_I_Average.hpp")],
        artwork: catalog_artwork("I Average", "18"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "19",
        title: "I Random",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/19_I_Random.hpp")],
        artwork: catalog_artwork("I Random", "19"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
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
        pack: "HP-67 Demonstration Pac I",
        reference: "21",
        title: "Dist Spd Time",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/21_DistSpdTime.hpp")],
        artwork: catalog_artwork("Dist Spd Time", "21"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "22",
        title: "Flags",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/22_Flags.hpp")],
        artwork: catalog_artwork("Flags", "22"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
        reference: "HP67diagA",
        title: "HP-67 Diagnostic A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/DemoPac1/HP67diagA.hpp")],
        artwork: catalog_artwork("HP-67 Diagnostic A", "HP67diagA"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Demonstration Pac I",
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
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-01A.hp67card")],
        artwork: catalog_artwork("Game of 21 (Blackjack)", "GA1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-02A",
        title: "Dice",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-02A.hp67card")],
        artwork: catalog_artwork("Dice", "GA1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-03A",
        title: "Slot Machine",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-03A.hp67card")],
        artwork: catalog_artwork("Slot Machine", "GA1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-04A",
        title: "Submarine Hunt",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-04A.hp67card")],
        artwork: catalog_artwork("Submarine Hunt", "GA1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-05A",
        title: "Artillery",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-05A.hp67card")],
        artwork: catalog_artwork("Artillery", "GA1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-06A1",
        title: "Space War 1",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-06A1.hp67card")],
        artwork: catalog_artwork("Space War 1", "GA1-06A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-06A2",
        title: "Space War 2",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-06A2.hp67card")],
        artwork: catalog_artwork("Space War 2", "GA1-06A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-07A",
        title: "Super Bagels",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-07A.hp67card")],
        artwork: catalog_artwork("Super Bagels", "GA1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-08A",
        title: "NIM",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-08A.hp67card")],
        artwork: catalog_artwork("NIM", "GA1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-09A",
        title: "Queen Board",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-09A.hp67card")],
        artwork: catalog_artwork("Queen Board", "GA1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-10A",
        title: "Hexapawn",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-10A.hp67card")],
        artwork: catalog_artwork("Hexapawn", "GA1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-11A",
        title: "Tic-Tac-Toe",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-11A.hp67card")],
        artwork: catalog_artwork("Tic-Tac-Toe", "GA1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-12A",
        title: "Wari",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-12A.hp67card")],
        artwork: catalog_artwork("Wari", "GA1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-13A",
        title: "Racetrack",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-13A.hp67card")],
        artwork: catalog_artwork("Racetrack", "GA1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-14A",
        title: "Teaser",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-14A.hp67card")],
        artwork: catalog_artwork("Teaser", "GA1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-15A",
        title: "Golf",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-15A.hp67card")],
        artwork: catalog_artwork("Golf", "GA1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-16A",
        title: "The Dealer",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-16A.hp67card")],
        artwork: catalog_artwork("The Dealer", "GA1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-17A",
        title: "Bowling Scorekeeper",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-17A.hp67card")],
        artwork: catalog_artwork("Bowling Scorekeeper", "GA1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-18A",
        title: "BioRhythms",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-18A.hp67card")],
        artwork: catalog_artwork("BioRhythms", "GA1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Games Pac I",
        reference: "GA1-19A",
        title: "Timer",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-games-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/games-20.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/games/GA1-19A.hp67card")],
        artwork: catalog_artwork("Timer", "GA1-19A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-01A",
        title: "Moving Average",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-01.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-01A.hp67card"
        )],
        artwork: catalog_artwork("Moving Average", "SD1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-02A",
        title: "Tabulator",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-02.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-02A.hp67card"
        )],
        artwork: catalog_artwork("Tabulator", "SD1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-03A",
        title: "Curve Fitting",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-03.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-03A.hp67card"
        )],
        artwork: catalog_artwork("Curve Fitting", "SD1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-04A",
        title: "Calendar Functions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-04.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-04A.hp67card"
        )],
        artwork: catalog_artwork("Calendar Functions", "SD1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-05A",
        title: "Annuities and Compound Amounts",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-05.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-05A.hp67card"
        )],
        artwork: catalog_artwork("Annuities and Compound Amounts", "SD1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-06A",
        title: "Follow Me",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-06A.hp67card"
        )],
        artwork: catalog_artwork("Follow Me", "SD1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-07A",
        title: "Triangle Solutions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-07.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-07A.hp67card"
        )],
        artwork: catalog_artwork("Triangle Solutions", "SD1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-08A",
        title: "Vector Operations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-08.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-08A.hp67card"
        )],
        artwork: catalog_artwork("Vector Operations", "SD1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-09A",
        title: "Polynomial Evaluation",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-09.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-09A.hp67card"
        )],
        artwork: catalog_artwork("Polynomial Evaluation", "SD1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-10A",
        title: "Matrix Operations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-10.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-10A.hp67card"
        )],
        artwork: catalog_artwork("Matrix Operations", "SD1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-11B",
        title: "Calculus and Roots of f(x)",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-11.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-11B.hp67card"
        )],
        artwork: catalog_artwork("Calculus and Roots of f(x)", "SD1-11B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-12A",
        title: "English-SI Conversions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-12.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-12A.hp67card"
        )],
        artwork: catalog_artwork("English-SI Conversions", "SD1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-13A",
        title: "Arithmetic Teacher",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-13.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-13A.hp67card"
        )],
        artwork: catalog_artwork("Arithmetic Teacher", "SD1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-14A",
        title: "Moon Rocket Lander",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-14.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-14A.hp67card"
        )],
        artwork: catalog_artwork("Moon Rocket Lander", "SD1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Standard Pac",
        reference: "SD1-15A",
        title: "Diagnostic Program",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/standard/SD1-15A.hp67card"
        )],
        artwork: catalog_artwork("Diagnostic Program", "SD1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Diagnostic Pac",
        reference: "SD-15C",
        title: "Diagnostic Program (SD-15C)",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp67-pac-standard-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/standard-15.png"),
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/HP-67 Diagnostic Cards/SD-15C-Diagnostic-Program.hp67card"
        ))],
        artwork: catalog_artwork("Diagnostic Program", "SD-15C"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-01",
        title: "Flow GSB/GTO/RTN — expect 7.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-01_Flow-GSB-GTO-RTN.hp67card"
        ))],
        artwork: catalog_artwork("Flow GSB/GTO/RTN", "CD-01"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-02",
        title: "Flags SF/CF/F? — expect 6.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-02_Flags-SF-CF-Test.hp67card"
        ))],
        artwork: catalog_artwork("Flags SF/CF/F?", "CD-02"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-03",
        title: "Conditionals — expect 7.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-03_Conditionals.hp67card"
        ))],
        artwork: catalog_artwork("Conditionals", "CD-03"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-04",
        title: "Indirect STO/RCL — expect 42.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-04_Indirect-STO-RCL.hp67card"
        ))],
        artwork: catalog_artwork("Indirect STO/RCL", "CD-04"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-05",
        title: "Nested GSB — expect 6.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-05_Nested-GSB.hp67card"
        ))],
        artwork: catalog_artwork("Nested GSB", "CD-05"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-06",
        title: "Second-half label search — expect 67.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-06_Second-Half-Label-Search.hp67card"
        ))],
        artwork: catalog_artwork("Second-half label search", "CD-06"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-07",
        title: "ISZ loop — expect 3.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-07_ISZ-Loop.hp67card"
        ))],
        artwork: catalog_artwork("ISZ loop", "CD-07"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-08",
        title: "DSZ loop — expect 3.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-08_DSZ-Loop.hp67card"
        ))],
        artwork: catalog_artwork("DSZ loop", "CD-08"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-09",
        title: "Long DSZ burn-in — expect 2000.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-09_Long-DSZ-Burn-In.hp67card"
        ))],
        artwork: catalog_artwork("Long DSZ burn-in", "CD-09"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-10",
        title: "Nested GSB burn-in — expect 500.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-10_Nested-GSB-Burn-In.hp67card"
        ))],
        artwork: catalog_artwork("Nested GSB burn-in", "CD-10"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-11",
        title: "Function identity burn-in — expect 1.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-11_Function-Identity-Burn-In.hp67card"
        ))],
        artwork: catalog_artwork("Function identity burn-in", "CD-11"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Custom Diagnostic Pac",
        reference: "CD-12",
        title: "Cross-half GSB burn-in — expect 500.00",
        source_pdf: None,
        artwork_path: None,
        parts: &[include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-12_Cross-Half-GSB-Burn-In.hp67card"
        ))],
        artwork: catalog_artwork("Cross-half GSB burn-in", "CD-12"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-01B",
        title: "Internal Rate Of Return",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-01.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-01B.hp67card"
        )],
        artwork: catalog_artwork("Internal Rate Of Return", "BD1-01B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-02B",
        title: "IRR--Groups",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-02.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-02B.hp67card"
        )],
        artwork: catalog_artwork("IRR--Groups", "BD1-02B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-03A",
        title: "DCF-NPV",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-03.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-03A.hp67card"
        )],
        artwork: catalog_artwork("DCF-NPV", "BD1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-04A",
        title: "Direct Reduction Loans,Sinking Fund",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-04.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-04A.hp67card"
        )],
        artwork: catalog_artwork("Direct Reduction Loans,Sinking Fund", "BD1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-05A",
        title: "Accumulated Int,Remaining Bal",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-05.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-05A.hp67card"
        )],
        artwork: catalog_artwork("Accumulated Int,Remaining Bal", "BD1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-06A",
        title: "Wrap-Around Mortgage",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-06A.hp67card"
        )],
        artwork: catalog_artwork("Wrap-Around Mortgage", "BD1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-07A",
        title: "Const Pmt To Principal",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-07.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-07A.hp67card"
        )],
        artwork: catalog_artwork("Const Pmt To Principal", "BD1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-08A",
        title: "Add-On,Rule Of 78's",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-08.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-08A.hp67card"
        )],
        artwork: catalog_artwork("Add-On,Rule Of 78's", "BD1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-09A",
        title: "Savings Plan - Leases",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-09.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-09A.hp67card"
        )],
        artwork: catalog_artwork("Savings Plan - Leases", "BD1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-10A",
        title: "Advance Payments",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-10.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-10A.hp67card"
        )],
        artwork: catalog_artwork("Advance Payments", "BD1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-11A",
        title: "Savings, Diff Per-Yr",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-11.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-11A.hp67card"
        )],
        artwork: catalog_artwork("Savings, Diff Per-Yr", "BD1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-12A",
        title: "Simple Int,Int Conv",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-12.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-12A.hp67card"
        )],
        artwork: catalog_artwork("Simple Int,Int Conv", "BD1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-13A",
        title: "Depreciation Schedules",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-13.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-13A.hp67card"
        )],
        artwork: catalog_artwork("Depreciation Schedules", "BD1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-14A",
        title: "Days Between Dates",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-14.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-14A.hp67card"
        )],
        artwork: catalog_artwork("Days Between Dates", "BD1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-15A",
        title: "Bond Price And Yield",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-15.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-15A.hp67card"
        )],
        artwork: catalog_artwork("Bond Price And Yield", "BD1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-16A",
        title: "Interest At Maturity,Discounted Securities",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-16.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-16A.hp67card"
        )],
        artwork: catalog_artwork("Interest At Maturity,Discounted Securities", "BD1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-17A",
        title: "Linear - Exponential",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-17.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-17A.hp67card"
        )],
        artwork: catalog_artwork("Linear - Exponential", "BD1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-18A",
        title: "Multiple Linear Regression",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-18.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-18A.hp67card"
        )],
        artwork: catalog_artwork("Multiple Linear Regression", "BD1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-19A",
        title: "Break-Even Analysis",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-19A.hp67card"
        )],
        artwork: catalog_artwork("Break-Even Analysis", "BD1-19A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-20A",
        title: "Invoicing",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-20.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-20A.hp67card"
        )],
        artwork: catalog_artwork("Invoicing", "BD1-20A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-21A",
        title: "Payroll",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-21.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-21A.hp67card"
        )],
        artwork: catalog_artwork("Payroll", "BD1-21A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Business Decisions Pac I",
        reference: "BD1-22A",
        title: "Inventory",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-business-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/business-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/business/BD1-22A.hp67card"
        )],
        artwork: catalog_artwork("Inventory", "BD1-22A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-01A",
        title: "Vector Statics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-01A.hp67card")],
        artwork: catalog_artwork("Vector Statics", "CE1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-02A1",
        title: "Section Properties - Input",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-02A1.hp67card")],
        artwork: catalog_artwork("Section Properties - Input", "CE1-02A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-02B2",
        title: "Section Properties - Output",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-02B2.hp67card")],
        artwork: catalog_artwork("Section Properties - Output", "CE1-02B2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-03A",
        title: "Properties of Special Sections",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-03A.hp67card")],
        artwork: catalog_artwork("Properties of Special Sections", "CE1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-04B",
        title: "Stress on an Element",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-04B.hp67card")],
        artwork: catalog_artwork("Stress on an Element", "CE1-04B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-05A",
        title: "Bending or Torsional Stress",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-05A.hp67card")],
        artwork: catalog_artwork("Bending or Torsional Stress", "CE1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-06A",
        title: "Linear or Angular Deformation",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-06A.hp67card")],
        artwork: catalog_artwork("Linear or Angular Deformation", "CE1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-07A",
        title: "Cantilever Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-07A.hp67card")],
        artwork: catalog_artwork("Cantilever Beams", "CE1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-08A",
        title: "Cantilever Beams - Trapezoidal Load",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-08A.hp67card")],
        artwork: catalog_artwork("Cantilever Beams - Trapezoidal Load", "CE1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-09A",
        title: "Simply Supported Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-09A.hp67card")],
        artwork: catalog_artwork("Simply Supported Beams", "CE1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-10A",
        title: "Simply Supported Beams - Trapezoidal Load",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-10A.hp67card")],
        artwork: catalog_artwork("Simply Supported Beams - Trapezoidal Load", "CE1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-11A",
        title: "Beams Fixed at Both Ends",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-11A.hp67card")],
        artwork: catalog_artwork("Beams Fixed at Both Ends", "CE1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-12A",
        title: "Beams Fixed at Both Ends - Trapezoidal Loads",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-12A.hp67card")],
        artwork: catalog_artwork("Beams Fixed at Both Ends - Trapezoidal Loads", "CE1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-13A",
        title: "Propped Cantilever Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-13A.hp67card")],
        artwork: catalog_artwork("Propped Cantilever Beams", "CE1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-14A",
        title: "Propped Cantilever Beams - Trapezoidal Load",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-14A.hp67card")],
        artwork: catalog_artwork("Propped Cantilever Beams - Trapezoidal Load", "CE1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-15A",
        title: "Six-span Continuous Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-15A.hp67card")],
        artwork: catalog_artwork("Six-span Continuous Beams", "CE1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-16A",
        title: "Steel Column Formula",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-16A.hp67card")],
        artwork: catalog_artwork("Steel Column Formula", "CE1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-17B",
        title: "Reinforced Concrete Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-17B.hp67card")],
        artwork: catalog_artwork("Reinforced Concrete Beams", "CE1-17B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Civil Engineering Pac I",
        reference: "CE1-18A",
        title: "Bolt Torque",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-civileng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/civil-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/civil/CE1-18A.hp67card")],
        artwork: catalog_artwork("Bolt Torque", "CE1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-01A1",
        title: "Network Transfer Functions - Input",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-01A1.hp67card")],
        artwork: catalog_artwork("Network Transfer Functions - Input", "EE1-01A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-01B2",
        title: "Network Transfer Functions - Output",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-01B2.hp67card")],
        artwork: catalog_artwork("Network Transfer Functions - Output", "EE1-01B2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-02A",
        title: "Reactive L-Network Impedance Matching",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-02A.hp67card")],
        artwork: catalog_artwork("Reactive L-Network Impedance Matching", "EE1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-03A",
        title: "Class A Transistor Amplifier Bias Optimization",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-03A.hp67card")],
        artwork: catalog_artwork("Class A Transistor Amplifier Bias Optimization", "EE1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-04A",
        title: "Transistor Amplifier Performance",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-04A.hp67card")],
        artwork: catalog_artwork("Transistor Amplifier Performance", "EE1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-05A",
        title: "Transistor Configuration Conversion",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-05A.hp67card")],
        artwork: catalog_artwork("Transistor Configuration Conversion", "EE1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-06A",
        title: "Parameter Conversion",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-06A.hp67card")],
        artwork: catalog_artwork("Parameter Conversion", "EE1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-07B",
        title: "Fourier Series",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-07B.hp67card")],
        artwork: catalog_artwork("Fourier Series", "EE1-07B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-08A",
        title: "Active Filter Design",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-08A.hp67card")],
        artwork: catalog_artwork("Active Filter Design", "EE1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-09A1",
        title: "Butterworth or Chebyshev Filter Design",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-09A1.hp67card")],
        artwork: catalog_artwork("Butterworth or Chebyshev Filter Design", "EE1-09A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-09A2",
        title: "Butterworth or Chebyshev Filter Design",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-09A2.hp67card")],
        artwork: catalog_artwork("Butterworth or Chebyshev Filter Design", "EE1-09A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-10A",
        title: "Bode Plot of Butterworth & Chebyshev Filters",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-10A.hp67card")],
        artwork: catalog_artwork("Bode Plot of Butterworth & Chebyshev Filters", "EE1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-11A",
        title: "Resistive Attenuator Design",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-11A.hp67card")],
        artwork: catalog_artwork("Resistive Attenuator Design", "EE1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-12A",
        title: "Smith Chart Conversions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-12A.hp67card")],
        artwork: catalog_artwork("Smith Chart Conversions", "EE1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-13A",
        title: "Transmission Line Impedance",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-13A.hp67card")],
        artwork: catalog_artwork("Transmission Line Impedance", "EE1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-14A",
        title: "Microstrip Calculations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-14A.hp67card")],
        artwork: catalog_artwork("Microstrip Calculations", "EE1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-15A",
        title: "Transmission Line Calculations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-15A.hp67card")],
        artwork: catalog_artwork("Transmission Line Calculations", "EE1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-16A",
        title: "Unilateral Design- Figure of Merit, Maximum Unilateral Gain, Gain Circles",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-16A.hp67card")],
        artwork: catalog_artwork(
            "Unilateral Design- Figure of Merit, Maximum Unilateral Gain, Gain Circles",
            "EE1-16A",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-17A",
        title: "Unilateral Design- Stability Factor, Maximum Gain, Optimum Matching",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-17A.hp67card")],
        artwork: catalog_artwork(
            "Unilateral Design- Stability Factor, Maximum Gain, Optimum Matching",
            "EE1-17A",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Electrical Engineering Pac I",
        reference: "EE1-18A",
        title: "Bilateral Design- Gain and Stability Circles",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-eleceng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/ee-20.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/ee/EE1-18A.hp67card")],
        artwork: catalog_artwork("Bilateral Design- Gain and Stability Circles", "EE1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-01A",
        title: "Factors and Primes",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-01A.hp67card")],
        artwork: catalog_artwork("Factors and Primes", "MA1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-02A",
        title: "GCD, LCM, Dec2Frac",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-02A.hp67card")],
        artwork: catalog_artwork("GCD, LCM, Dec2Frac", "MA1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-03A",
        title: "Base Conversions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-03A.hp67card")],
        artwork: catalog_artwork("Base Conversions", "MA1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-04A",
        title: "Optimal Scale, Plot",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-04A.hp67card")],
        artwork: catalog_artwork("Optimal Scale, Plot", "MA1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-05A",
        title: "Complex Operations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-05A.hp67card")],
        artwork: catalog_artwork("Complex Operations", "MA1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-06A",
        title: "Polynomial Solutions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-06A.hp67card")],
        artwork: catalog_artwork("Polynomial Solutions", "MA1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-07A1",
        title: "4x4 Matrix Setup",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-07A1.hp67card")],
        artwork: catalog_artwork("4x4 Matrix Setup", "MA1-07A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-07A2",
        title: "4x4 Matrix Solutions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-07A2.hp67card")],
        artwork: catalog_artwork("4x4 Matrix Solutions", "MA1-07A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-08A",
        title: "Solution to f(x)",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-08A.hp67card")],
        artwork: catalog_artwork("Solution to f(x)", "MA1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-09A",
        title: "Numerical Integratio",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-09A.hp67card")],
        artwork: catalog_artwork("Numerical Integratio", "MA1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-10A",
        title: "Gaussian Quadrature",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-10A.hp67card")],
        artwork: catalog_artwork("Gaussian Quadrature", "MA1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-11A",
        title: "Differential Equatio",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-11A.hp67card")],
        artwork: catalog_artwork("Differential Equatio", "MA1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-12A",
        title: "Interpolations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-12A.hp67card")],
        artwork: catalog_artwork("Interpolations", "MA1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-13A",
        title: "Coord. Transforms",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-13A.hp67card")],
        artwork: catalog_artwork("Coord. Transforms", "MA1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-14A",
        title: "Intersections",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-14A.hp67card")],
        artwork: catalog_artwork("Intersections", "MA1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-15A",
        title: "Circles",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-15A.hp67card")],
        artwork: catalog_artwork("Circles", "MA1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-16A",
        title: "Spherical Triangles",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-16A.hp67card")],
        artwork: catalog_artwork("Spherical Triangles", "MA1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-17A",
        title: "Gamma Function",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-17A.hp67card")],
        artwork: catalog_artwork("Gamma Function", "MA1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-18A",
        title: "Bessel Functions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-18A.hp67card")],
        artwork: catalog_artwork("Bessel Functions", "MA1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mathematics Pac I",
        reference: "MA1-19A",
        title: "Hyperbolics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-math-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/math-20.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/math/MA1-19A.hp67card")],
        artwork: catalog_artwork("Hyperbolics", "MA1-19A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-01A1",
        title: "Vector Statics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-01A1.hp67card")],
        artwork: catalog_artwork("Vector Statics", "ME1-01A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-02A1",
        title: "Section Properties - Input",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-02A1.hp67card")],
        artwork: catalog_artwork("Section Properties - Input", "ME1-02A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-02A2",
        title: "Section Properties - Output",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-02A2.hp67card")],
        artwork: catalog_artwork("Section Properties - Output", "ME1-02A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-03A",
        title: "Stress On An Element",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-03A.hp67card")],
        artwork: catalog_artwork("Stress On An Element", "ME1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-04A",
        title: "Soderberg's Equation For Fatigue",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-04A.hp67card")],
        artwork: catalog_artwork("Soderberg's Equation For Fatigue", "ME1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-05A",
        title: "Cantilever Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-05A.hp67card")],
        artwork: catalog_artwork("Cantilever Beams", "ME1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-06A",
        title: "Simply Supported Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-06A.hp67card")],
        artwork: catalog_artwork("Simply Supported Beams", "ME1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-07A",
        title: "Beams Fixed At Both Ends",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-07A.hp67card")],
        artwork: catalog_artwork("Beams Fixed At Both Ends", "ME1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-08A",
        title: "Propped Cantilever Beams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-08A.hp67card")],
        artwork: catalog_artwork("Propped Cantilever Beams", "ME1-08A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-09A",
        title: "Helical Spring Design",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-09A.hp67card")],
        artwork: catalog_artwork("Helical Spring Design", "ME1-09A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-10A1",
        title: "Four Bar Function Generator",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-10A1.hp67card")],
        artwork: catalog_artwork("Four Bar Function Generator", "ME1-10A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-10A2",
        title: "Four Bar Function Generator",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-10A2.hp67card")],
        artwork: catalog_artwork("Four Bar Function Generator", "ME1-10A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-11A",
        title: "Progression of Four-Bar System",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-11A.hp67card")],
        artwork: catalog_artwork("Progression of Four-Bar System", "ME1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-12A",
        title: "Progression Of Slider Crank",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-12A.hp67card")],
        artwork: catalog_artwork("Progression Of Slider Crank", "ME1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-13A",
        title: "Circular Cams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-13A.hp67card")],
        artwork: catalog_artwork("Circular Cams", "ME1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-14A",
        title: "Linear Cams",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-14A.hp67card")],
        artwork: catalog_artwork("Linear Cams", "ME1-14A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-15A",
        title: "Gear Forces",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-15A.hp67card")],
        artwork: catalog_artwork("Gear Forces", "ME1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-16A",
        title: "Standard External Involute Spur Gears",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-16A.hp67card")],
        artwork: catalog_artwork("Standard External Involute Spur Gears", "ME1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-17A",
        title: "Belt Length",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-17A.hp67card")],
        artwork: catalog_artwork("Belt Length", "ME1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-18A",
        title: "Free Vibrations",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-20.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-18A.hp67card")],
        artwork: catalog_artwork("Free Vibrations", "ME1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-19B",
        title: "Vibrations Forced By F0COSwt",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-21.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-19B.hp67card")],
        artwork: catalog_artwork("Vibrations Forced By F0COSwt", "ME1-19B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-20A",
        title: "Equations Of State",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-22.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-20A.hp67card")],
        artwork: catalog_artwork("Equations Of State", "ME1-20A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-21A",
        title: "Isentropic Flow For Ideal Gases",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-23.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-21A.hp67card")],
        artwork: catalog_artwork("Isentropic Flow For Ideal Gases", "ME1-21A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-22A",
        title: "Conduit Flow",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-24.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-22A.hp67card")],
        artwork: catalog_artwork("Conduit Flow", "ME1-22A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-23A1",
        title: "Heat Exchangers, Parallel-Flow and Counter-Flow",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-25.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-23A1.hp67card")],
        artwork: catalog_artwork(
            "Heat Exchangers, Parallel-Flow and Counter-Flow",
            "ME1-23A1",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Mechanical Engineering Pac I",
        reference: "ME1-23A2",
        title: "Parallel Counter-Flow",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-mecheng-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/me-26.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/me/ME1-23A2.hp67card")],
        artwork: catalog_artwork("Parallel Counter-Flow", "ME1-23A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-01A",
        title: "Basic Stat for 2 Var",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-01.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-01A.hp67card")],
        artwork: catalog_artwork("Basic Stat for 2 Var", "ST1-01A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-02A",
        title: "Factorial, Permutati",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-02.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-02A.hp67card")],
        artwork: catalog_artwork("Factorial, Permutati", "ST1-02A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-03A",
        title: "Moments, Skewness, K",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-03.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-03A.hp67card")],
        artwork: catalog_artwork("Moments, Skewness, K", "ST1-03A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-04A",
        title: "Random Number Genera",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-04.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-04A.hp67card")],
        artwork: catalog_artwork("Random Number Genera", "ST1-04A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-05A",
        title: "Histogram",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-05.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-05A.hp67card")],
        artwork: catalog_artwork("Histogram", "ST1-05A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-06A",
        title: "Variance, One Way",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-06.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-06A.hp67card")],
        artwork: catalog_artwork("Variance, One Way", "ST1-06A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-07A",
        title: "Variance, Two Way",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-07.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-07A.hp67card")],
        artwork: catalog_artwork("Variance, Two Way", "ST1-07A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-08A1",
        title: "Covariance, One Way",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-08.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-08A1.hp67card")],
        artwork: catalog_artwork("Covariance, One Way", "ST1-08A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-08A2",
        title: "Covariance, One Way",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-09.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-08A2.hp67card")],
        artwork: catalog_artwork("Covariance, One Way", "ST1-08A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-09A1",
        title: "Normal Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-10.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-09A1.hp67card")],
        artwork: catalog_artwork("Normal Distribution", "ST1-09A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-09A2",
        title: "Normal Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-11.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-09A2.hp67card")],
        artwork: catalog_artwork("Normal Distribution", "ST1-09A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-10A",
        title: "Chi-Square Distribut",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-12.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-10A.hp67card")],
        artwork: catalog_artwork("Chi-Square Distribut", "ST1-10A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-11A",
        title: "t Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-13.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-11A.hp67card")],
        artwork: catalog_artwork("t Distribution", "ST1-11A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-12A",
        title: "F Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-14.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-12A.hp67card")],
        artwork: catalog_artwork("F Distribution", "ST1-12A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-13A",
        title: "Multiple Linear Regr",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-15.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-13A.hp67card")],
        artwork: catalog_artwork("Multiple Linear Regr", "ST1-13A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-14A1",
        title: "Polynomial Approxima",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-16.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-14A1.hp67card")],
        artwork: catalog_artwork("Polynomial Approxima", "ST1-14A1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-14A2",
        title: "Polynomial Approxima",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-17.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-14A2.hp67card")],
        artwork: catalog_artwork("Polynomial Approxima", "ST1-14A2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-15A",
        title: "t Statistics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-18.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-15A.hp67card")],
        artwork: catalog_artwork("t Statistics", "ST1-15A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-16A",
        title: "Chi-Square Evaluatio",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-19.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-16A.hp67card")],
        artwork: catalog_artwork("Chi-Square Evaluatio", "ST1-16A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-17A",
        title: "Contingency Table",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-20.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-17A.hp67card")],
        artwork: catalog_artwork("Contingency Table", "ST1-17A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-18A",
        title: "Spearman's RankCorre",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-21.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-18A.hp67card")],
        artwork: catalog_artwork("Spearman's RankCorre", "ST1-18A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-19A",
        title: "x & R Control Charts",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-22.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-19A.hp67card")],
        artwork: catalog_artwork("x & R Control Charts", "ST1-19A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-20A",
        title: "Operating Characteri",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-23.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-20A.hp67card")],
        artwork: catalog_artwork("Operating Characteri", "ST1-20A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Statistics Pac I",
        reference: "ST1-21A",
        title: "Server Queues",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-stat-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/stat-24.png"),
        parts: &[hpp!("/programs/HP67/ImportedPACs/stat/ST1-21A.hp67card")],
        artwork: catalog_artwork("Server Queues", "ST1-21A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-01-ee5c4dc6",
        title: "01 - Beer's Law",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-01.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-01-ee5c4dc6.hp67card"
        )],
        artwork: catalog_artwork("01 - Beer's Law", "CL1-01-ee5c4dc6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-02-ff6f6413",
        title: "02 - Protein Electrophoresis",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-02.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-02-ff6f6413.hp67card"
        )],
        artwork: catalog_artwork("02 - Protein Electrophoresis", "CL1-02-ff6f6413"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-03-577a1315",
        title: "03 - LDH Isoenzymes",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-03.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-03-577a1315.hp67card"
        )],
        artwork: catalog_artwork("03 - LDH Isoenzymes", "CL1-03-577a1315"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-04-d8c0d74e",
        title: "04 - Body Surface Area",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-04.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-04-d8c0d74e.hp67card"
        )],
        artwork: catalog_artwork("04 - Body Surface Area", "CL1-04-d8c0d74e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-05-4d38456e",
        title: "05 - Urea Clearance",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-05.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-05-4d38456e.hp67card"
        )],
        artwork: catalog_artwork("05 - Urea Clearance", "CL1-05-4d38456e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-06-b8cb5a4f",
        title: "06 - Creatinine Clearance",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-06-b8cb5a4f.hp67card"
        )],
        artwork: catalog_artwork("06 - Creatinine Clearance", "CL1-06-b8cb5a4f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-07-ed7f2e70",
        title: "07 - Amniotic Fluid Assay",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-07.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-07-ed7f2e70.hp67card"
        )],
        artwork: catalog_artwork("07 - Amniotic Fluid Assay", "CL1-07-ed7f2e70"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-08-c75972b4",
        title: "08 - Blood Acid-Base Status",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-08.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-08-c75972b4.hp67card"
        )],
        artwork: catalog_artwork("08 - Blood Acid-Base Status", "CL1-08-c75972b4"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-09-afcecdc8",
        title: "09 - Oxygen Saturation and Content",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-09.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-09-afcecdc8.hp67card"
        )],
        artwork: catalog_artwork("09 - Oxygen Saturation and Content", "CL1-09-afcecdc8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-10-b5d7a909",
        title: "10 - Red Cell Indices",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-10.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-10-b5d7a909.hp67card"
        )],
        artwork: catalog_artwork("10 - Red Cell Indices", "CL1-10-b5d7a909"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-11-4db0ca8a",
        title: "11 - Total Blood Volume",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-11.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-11-4db0ca8a.hp67card"
        )],
        artwork: catalog_artwork("11 - Total Blood Volume", "CL1-11-4db0ca8a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-12-33ae4423",
        title: "12 - Schilling Test",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-12.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-12-33ae4423.hp67card"
        )],
        artwork: catalog_artwork("12 - Schilling Test", "CL1-12-33ae4423"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-13-4cccf7c3",
        title: "13 - Thyroid Uptake",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-13.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-13-4cccf7c3.hp67card"
        )],
        artwork: catalog_artwork("13 - Thyroid Uptake", "CL1-13-4cccf7c3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-14-cc726078",
        title: "14 - Radioactive Decay Corrections",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-14.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-14-cc726078.hp67card"
        )],
        artwork: catalog_artwork("14 - Radioactive Decay Corrections", "CL1-14-cc726078"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-15-4befaa17",
        title: "15 - Radioimmunoassay",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-15.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-15-4befaa17.hp67card"
        )],
        artwork: catalog_artwork("15 - Radioimmunoassay", "CL1-15-4befaa17"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-16-7d440be8",
        title: "16 - Basic Statistics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-16.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-16-7d440be8.hp67card"
        )],
        artwork: catalog_artwork("16 - Basic Statistics", "CL1-16-7d440be8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-17-7ea1b500",
        title: "17 - Chi-Square Evaluation and Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-17.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-17-7ea1b500.hp67card"
        )],
        artwork: catalog_artwork(
            "17 - Chi-Square Evaluation and Distribution",
            "CL1-17-7ea1b500",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-18-ab9c2274",
        title: "18 - t Statistics",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-18.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-18-ab9c2274.hp67card"
        )],
        artwork: catalog_artwork("18 - t Statistics", "CL1-18-ab9c2274"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        reference: "CL1-19-a3413ecd",
        title: "19 - t Distribution",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-clinical-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/clinical-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/clinical/CL1-19-a3413ecd.hp67card"
        )],
        artwork: catalog_artwork("19 - t Distribution", "CL1-19-a3413ecd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-01-11cf36c0",
        title: "01 - Estimated Time of Arrival",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-01.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-01-11cf36c0.hp67card"
        )],
        artwork: catalog_artwork("01 - Estimated Time of Arrival", "NAV-01-11cf36c0"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-02-182573cd",
        title: "02 - Great Circle and Rhumb Line Navigation",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-02.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-02-182573cd.hp67card"
        )],
        artwork: catalog_artwork(
            "02 - Great Circle and Rhumb Line Navigation",
            "NAV-02-182573cd",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-03-e30993d3",
        title: "03 - Dead Reckoning",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-03.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-03-e30993d3.hp67card"
        )],
        artwork: catalog_artwork("03 - Dead Reckoning", "NAV-03-e30993d3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-04-3bcb6e42",
        title: "04 - Velocity Triangle and Course to Steer",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-04.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-04-3bcb6e42.hp67card"
        )],
        artwork: catalog_artwork(
            "04 - Velocity Triangle and Course to Steer",
            "NAV-04-3bcb6e42",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-05-6d0d4ae7",
        title: "05 - Star Sight Planner",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-05.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-05-6d0d4ae7.hp67card"
        )],
        artwork: catalog_artwork("05 - Star Sight Planner", "NAV-05-6d0d4ae7"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-05-D-488f90d2",
        title: "NAV-05A2",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-05-D-488f90d2.hp67card"
        )],
        artwork: catalog_artwork("NAV-05A2", "NAV-05-D-488f90d2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-05-D-af8a9ee4",
        title: "NAV-05A2",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-05-D-af8a9ee4.hp67card"
        )],
        artwork: catalog_artwork("NAV-05A2", "NAV-05-D-af8a9ee4"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-06-12033ab1",
        title: "06 - Almanac Interpolator",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-07.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-06-12033ab1.hp67card"
        )],
        artwork: catalog_artwork("06 - Almanac Interpolator", "NAV-06-12033ab1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-07-751f9bb7",
        title: "07 - Sun Line of Position",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-08.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-07-751f9bb7.hp67card"
        )],
        artwork: catalog_artwork("07 - Sun Line of Position", "NAV-07-751f9bb7"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-3dc76442",
        title: "08 - Star Line of Position",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-09.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-3dc76442.hp67card"
        )],
        artwork: catalog_artwork("08 - Star Line of Position", "NAV-08-3dc76442"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-0db449e9",
        title: "08a - Star Data 1",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-10.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-0db449e9.hp67card"
        )],
        artwork: catalog_artwork("08a - Star Data 1", "NAV-08-0db449e9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-6b06eea3",
        title: "08b - Star Data 2",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-11.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-6b06eea3.hp67card"
        )],
        artwork: catalog_artwork("08b - Star Data 2", "NAV-08-6b06eea3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-79aa6f37",
        title: "08c - Star Data 3",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-12.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-79aa6f37.hp67card"
        )],
        artwork: catalog_artwork("08c - Star Data 3", "NAV-08-79aa6f37"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-8ea27fd1",
        title: "08d - Star Data 4",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-13.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-8ea27fd1.hp67card"
        )],
        artwork: catalog_artwork("08d - Star Data 4", "NAV-08-8ea27fd1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-b6ccb381",
        title: "08e - Star Data 5",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-14.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-b6ccb381.hp67card"
        )],
        artwork: catalog_artwork("08e - Star Data 5", "NAV-08-b6ccb381"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-08-2d858830",
        title: "08f - Star Data 6",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-15.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-08-2d858830.hp67card"
        )],
        artwork: catalog_artwork("08f - Star Data 6", "NAV-08-2d858830"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-09-7741f37b",
        title: "09 - Bearing Line of Position",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-16.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-09-7741f37b.hp67card"
        )],
        artwork: catalog_artwork("09 - Bearing Line of Position", "NAV-09-7741f37b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-10-8906d17d",
        title: "10 - Two-Angle Line of Position",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-17.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-10-8906d17d.hp67card"
        )],
        artwork: catalog_artwork("10 - Two-Angle Line of Position", "NAV-10-8906d17d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-11-b47eee2c",
        title: "11 - Fix From Two Lines of Position",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-18.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-11-b47eee2c.hp67card"
        )],
        artwork: catalog_artwork("11 - Fix From Two Lines of Position", "NAV-11-b47eee2c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-12-b10aa27b",
        title: "12 - Distance by Horizon Angle",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-12-b10aa27b.hp67card"
        )],
        artwork: catalog_artwork("12 - Distance by Horizon Angle", "NAV-12-b10aa27b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-13-61f3a634",
        title: "13 - Radar Plotting - Closest Point of Approach",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-20.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-13-61f3a634.hp67card"
        )],
        artwork: catalog_artwork(
            "13 - Radar Plotting - Closest Point of Approach",
            "NAV-13-61f3a634",
        ),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Navigation Pac I",
        reference: "NAV-14-ae9cf5f6",
        title: "14 - Beating to Windward",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-navigation-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/navigation-21.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/navigation/NAV-14-ae9cf5f6.hp67card"
        )],
        artwork: catalog_artwork("14 - Beating to Windward", "NAV-14-ae9cf5f6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-01-7620b7c6",
        title: "01 - Traverse, Inverse and Sideshots",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-01.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-01-7620b7c6.hp67card"
        )],
        artwork: catalog_artwork("01 - Traverse, Inverse and Sideshots", "SU1-01-7620b7c6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-02-a1882dbf",
        title: "02 - Traverse Adjustment",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-02.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-02-a1882dbf.hp67card"
        )],
        artwork: catalog_artwork("02 - Traverse Adjustment", "SU1-02-a1882dbf"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-03-efd5b55b",
        title: "03 - Intersections",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-03.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-03-efd5b55b.hp67card"
        )],
        artwork: catalog_artwork("03 - Intersections", "SU1-03-efd5b55b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-04-4501fc83",
        title: "04 - Curve Solutions",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-04.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-04-4501fc83.hp67card"
        )],
        artwork: catalog_artwork("04 - Curve Solutions", "SU1-04-4501fc83"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-05-72451190",
        title: "05 - Horizontal Curve Layout",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-05.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-05-72451190.hp67card"
        )],
        artwork: catalog_artwork("05 - Horizontal Curve Layout", "SU1-05-72451190"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-06-ac0565ca",
        title: "06 - Spiral Curve Layout",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-06.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-06-ac0565ca.hp67card"
        )],
        artwork: catalog_artwork("06 - Spiral Curve Layout", "SU1-06-ac0565ca"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-07-7d87d22c",
        title: "07 - Vertical Curves and Grades",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-07.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-07-7d87d22c.hp67card"
        )],
        artwork: catalog_artwork("07 - Vertical Curves and Grades", "SU1-07-7d87d22c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-08-dbd9ff14",
        title: "08 - Resection",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-08.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-08-dbd9ff14.hp67card"
        )],
        artwork: catalog_artwork("08 - Resection", "SU1-08-dbd9ff14"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-09-08b9e847",
        title: "09 - Two Instrument Radial Survey",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-09.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-09-08b9e847.hp67card"
        )],
        artwork: catalog_artwork("09 - Two Instrument Radial Survey", "SU1-09-08b9e847"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-10-5685c1d9",
        title: "10 - EDM Slope Reduction",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-10.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-10-5685c1d9.hp67card"
        )],
        artwork: catalog_artwork("10 - EDM Slope Reduction", "SU1-10-5685c1d9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-11-b38e2503",
        title: "11 - Stadia Reduction/3-Wire Leveling",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-11.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-11-b38e2503.hp67card"
        )],
        artwork: catalog_artwork("11 - Stadia Reduction/3-Wire Leveling", "SU1-11-b38e2503"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-12-92762b3c",
        title: "12 - Taping Reduction/Field Angle Check",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-12.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-12-92762b3c.hp67card"
        )],
        artwork: catalog_artwork("12 - Taping Reduction/Field Angle Check", "SU1-12-92762b3c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-13-9a26723d",
        title: "13 - Azimuth of the Sun",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-13.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-13-9a26723d.hp67card"
        )],
        artwork: catalog_artwork("13 - Azimuth of the Sun", "SU1-13-9a26723d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-14-2115b409",
        title: "14 - Predetermined Area",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-14.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-14-2115b409.hp67card"
        )],
        artwork: catalog_artwork("14 - Predetermined Area", "SU1-14-2115b409"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-15-784a11db",
        title: "15a - Earthwork (original)",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-15.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-15-784a11db.hp67card"
        )],
        artwork: catalog_artwork("15a - Earthwork (original)", "SU1-15-784a11db"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-15-2e25d058",
        title: "15b - Earthwork (improved)",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-15.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-15-2e25d058.hp67card"
        )],
        artwork: catalog_artwork("15b - Earthwork (improved)", "SU1-15-2e25d058"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-16-8c5158b3",
        title: "16 - Coordinate Transformation",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-16.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-16-8c5158b3.hp67card"
        )],
        artwork: catalog_artwork("16 - Coordinate Transformation", "SU1-16-8c5158b3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-a4848e5c",
        title: "Lambert Data Alaska",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-a4848e5c.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Alaska", "SU1-17-D-a4848e5c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-50de669b",
        title: "17a - Geographic to Lambert",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-17.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-50de669b.hp67card"
        )],
        artwork: catalog_artwork("17a - Geographic to Lambert", "SU1-17-50de669b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-63bb3fe9",
        title: "17b - Lambert to Geographic",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-18.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-63bb3fe9.hp67card"
        )],
        artwork: catalog_artwork("17b - Lambert to Geographic", "SU1-17-63bb3fe9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-965271bd",
        title: "Lambert Data Alaska",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-965271bd.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Alaska", "SU1-17-D-965271bd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-60f4dfdd",
        title: "Lambert Data Arkansas North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-60f4dfdd.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Arkansas North", "SU1-17-D-60f4dfdd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-4f3a3be6",
        title: "Lambert Data Arkansas South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-4f3a3be6.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Arkansas South", "SU1-17-D-4f3a3be6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-fcc0ec19",
        title: "Lambert Data California I",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-fcc0ec19.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California I", "SU1-17-D-fcc0ec19"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-806db91f",
        title: "Lambert Data California II",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-806db91f.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California II", "SU1-17-D-806db91f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-bfece0c8",
        title: "Lambert Data California IV",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-bfece0c8.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California IV", "SU1-17-D-bfece0c8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-8f05812f",
        title: "Lambert Data California V",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-8f05812f.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California V", "SU1-17-D-8f05812f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-4155b930",
        title: "Lambert Data California VI",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-4155b930.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California VI", "SU1-17-D-4155b930"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-34fe1da2",
        title: "Lambert Data California VII",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-34fe1da2.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data California VII", "SU1-17-D-34fe1da2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-b205f584",
        title: "Lambert Data Colorado Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-b205f584.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Colorado Central", "SU1-17-D-b205f584"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-920a1479",
        title: "Lambert Data Colorado North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-920a1479.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Colorado North", "SU1-17-D-920a1479"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-7d2e871a",
        title: "Lambert Data Colorado South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-7d2e871a.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Colorado South", "SU1-17-D-7d2e871a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-a7df7a8f",
        title: "Lambert Data Connecticut",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-a7df7a8f.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Connecticut", "SU1-17-D-a7df7a8f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-74847e7e",
        title: "Lambert Data Florida North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-74847e7e.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Florida North", "SU1-17-D-74847e7e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-49dcc1db",
        title: "Lambert Data Iowa North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-49dcc1db.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Iowa North", "SU1-17-D-49dcc1db"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-b1c20e00",
        title: "Lambert Data Kansas North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-b1c20e00.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Kansas North", "SU1-17-D-b1c20e00"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-f7c5fc6d",
        title: "Lambert Data Kansas South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-f7c5fc6d.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Kansas South", "SU1-17-D-f7c5fc6d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-8ab84ec6",
        title: "Lambert Data Kentucky North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-8ab84ec6.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Kentucky North", "SU1-17-D-8ab84ec6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-330f8da6",
        title: "Lambert Data Kentucky South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-330f8da6.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Kentucky South", "SU1-17-D-330f8da6"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-ba23b118",
        title: "Lambert Data Louisiana North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-ba23b118.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Louisiana North", "SU1-17-D-ba23b118"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-26d7e2aa",
        title: "Lambert Data Louisiana Offshore",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-26d7e2aa.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Louisiana Offshore", "SU1-17-D-26d7e2aa"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-c09c7b87",
        title: "Lambert Data Louisiana South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-c09c7b87.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Louisiana South", "SU1-17-D-c09c7b87"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-dfe2927e",
        title: "Lambert Data Maryland",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-dfe2927e.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Maryland", "SU1-17-D-dfe2927e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-c2b24f1a",
        title: "Lambert Data Massachusetts Island",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-c2b24f1a.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Massachusetts Island", "SU1-17-D-c2b24f1a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-4be35b97",
        title: "Lambert Data Massachusetts Mainland",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-4be35b97.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Massachusetts Mainland", "SU1-17-D-4be35b97"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-0451902b",
        title: "Lambert Data Michigan Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-0451902b.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Michigan Central", "SU1-17-D-0451902b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-fa4f5447",
        title: "Lambert Data Michigan North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-fa4f5447.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Michigan North", "SU1-17-D-fa4f5447"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-35248266",
        title: "Lambert Data Michigan South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-35248266.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Michigan South", "SU1-17-D-35248266"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-a823f8d4",
        title: "Lambert Data Minnesota Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-a823f8d4.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Minnesota Central", "SU1-17-D-a823f8d4"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-25e703dd",
        title: "Lambert Data Minnesota North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-25e703dd.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Minnesota North", "SU1-17-D-25e703dd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-578f6c07",
        title: "Lambert Data Minnesota South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-578f6c07.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Minnesota South", "SU1-17-D-578f6c07"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-fd5fc2f8",
        title: "Lambert Data Montana Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-fd5fc2f8.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Montana Central", "SU1-17-D-fd5fc2f8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-87cfc74f",
        title: "Lambert Data Montana North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-87cfc74f.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Montana North", "SU1-17-D-87cfc74f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-dc620c78",
        title: "Lambert Data Nebraska North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-dc620c78.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Nebraska North", "SU1-17-D-dc620c78"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-999aa537",
        title: "Lambert Data Nebraska South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-999aa537.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Nebraska South", "SU1-17-D-999aa537"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-5fab8485",
        title: "Lambert Data New York Long Island",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-5fab8485.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data New York Long Island", "SU1-17-D-5fab8485"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-482ed621",
        title: "Lambert Data North Carolina",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-482ed621.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data North Carolina", "SU1-17-D-482ed621"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-ee3280c1",
        title: "Lambert Data North Dakota North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-ee3280c1.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data North Dakota North", "SU1-17-D-ee3280c1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-36ede5f9",
        title: "Lambert Data North Dakota South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-36ede5f9.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data North Dakota South", "SU1-17-D-36ede5f9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-dd69b774",
        title: "Lambert Data Ohio North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-dd69b774.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Ohio North", "SU1-17-D-dd69b774"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-b045f1cd",
        title: "Lambert Data Ohio South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-b045f1cd.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Ohio South", "SU1-17-D-b045f1cd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-e15eb09c",
        title: "Lambert Data Oklahoma North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-e15eb09c.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Oklahoma North", "SU1-17-D-e15eb09c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-45ef7ca0",
        title: "Lambert Data Oklahoma South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-45ef7ca0.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Oklahoma South", "SU1-17-D-45ef7ca0"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-0b9d7685",
        title: "Lambert Data Oregon North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-0b9d7685.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Oregon North", "SU1-17-D-0b9d7685"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-25e4c936",
        title: "Lambert Data Oregon South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-25e4c936.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Oregon South", "SU1-17-D-25e4c936"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-55a528d2",
        title: "Lambert Data Pennsylvania North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-55a528d2.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Pennsylvania North", "SU1-17-D-55a528d2"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-75cce508",
        title: "Lambert Data Pennsylvania South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-75cce508.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Pennsylvania South", "SU1-17-D-75cce508"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-4457215b",
        title: "Lambert Data South Carolina North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-4457215b.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data South Carolina North", "SU1-17-D-4457215b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-7d9e8ede",
        title: "Lambert Data South Dakota North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-7d9e8ede.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data South Dakota North", "SU1-17-D-7d9e8ede"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-f07f3480",
        title: "Lambert Data South Dakota South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-f07f3480.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data South Dakota South", "SU1-17-D-f07f3480"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-c5bb8bea",
        title: "Lambert Data Tennessee",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-c5bb8bea.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Tennessee", "SU1-17-D-c5bb8bea"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-90feaf64",
        title: "Lambert Data Texas Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-90feaf64.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Texas Central", "SU1-17-D-90feaf64"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-c697f9d1",
        title: "Lambert Data Texas North Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-c697f9d1.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Texas North Central", "SU1-17-D-c697f9d1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-11563bbd",
        title: "Lambert Data Texas North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-11563bbd.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Texas North", "SU1-17-D-11563bbd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-2a7eeb4c",
        title: "Lambert Data Texas South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-2a7eeb4c.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Texas South", "SU1-17-D-2a7eeb4c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-1ffbcefc",
        title: "Lambert Data Utah Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-1ffbcefc.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Utah Central", "SU1-17-D-1ffbcefc"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-e932489a",
        title: "Lambert Data Utah North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-e932489a.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Utah North", "SU1-17-D-e932489a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-0b8ad8d5",
        title: "Lambert Data Utah South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-0b8ad8d5.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Utah South", "SU1-17-D-0b8ad8d5"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-7c8fc840",
        title: "Lambert Data Virginia North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-7c8fc840.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Virginia North", "SU1-17-D-7c8fc840"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-d9b30417",
        title: "Lambert Data Virginia South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-d9b30417.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Virginia South", "SU1-17-D-d9b30417"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-e6c85063",
        title: "Lambert Data Washington South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-e6c85063.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Washington South", "SU1-17-D-e6c85063"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-50da60f9",
        title: "Lambert Data West Virginia North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-50da60f9.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data West Virginia North", "SU1-17-D-50da60f9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-ffbd2ccb",
        title: "Lambert Data West Virginia South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-ffbd2ccb.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data West Virginia South", "SU1-17-D-ffbd2ccb"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-d72aac40",
        title: "Lambert Data Wisconsin Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-d72aac40.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Wisconsin Central", "SU1-17-D-d72aac40"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-6188761e",
        title: "Lambert Data Wisconsin North",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-6188761e.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Wisconsin North", "SU1-17-D-6188761e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-17-D-7a3c9074",
        title: "Lambert Data Wisconsin South",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-19.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-17-D-7a3c9074.hp67card"
        )],
        artwork: catalog_artwork("Lambert Data Wisconsin South", "SU1-17-D-7a3c9074"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-30e57121",
        title: "Mercator Data Idaho West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-30e57121.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Idaho West", "SU1-18-D-30e57121"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-40f333d8",
        title: "18a - Geographic to Mercator",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-20.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-40f333d8.hp67card"
        )],
        artwork: catalog_artwork("18a - Geographic to Mercator", "SU1-18-40f333d8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-1d7b2a6a",
        title: "18b - Mercator to Geographic",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-21.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-1d7b2a6a.hp67card"
        )],
        artwork: catalog_artwork("18b - Mercator to Geographic", "SU1-18-1d7b2a6a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-734575da",
        title: "Mercator Data Alabama West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-734575da.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Alabama West", "SU1-18-D-734575da"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-fc36e803",
        title: "Mercator Data Alabama East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-fc36e803.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Alabama East", "SU1-18-D-fc36e803"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-f56a60a4",
        title: "Mercator Data Arizona Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-f56a60a4.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Arizona Central", "SU1-18-D-f56a60a4"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-f12f6946",
        title: "Mercator Data Arizona East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-f12f6946.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Arizona East", "SU1-18-D-f12f6946"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-05308988",
        title: "Mercator Data Arizona West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-05308988.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Arizona West", "SU1-18-D-05308988"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-1cf9269a",
        title: "Mercator Data Delaware",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-1cf9269a.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Delaware", "SU1-18-D-1cf9269a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-0e899821",
        title: "Mercator Data Florida East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-0e899821.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Florida East", "SU1-18-D-0e899821"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-d881f713",
        title: "Mercator Data Florida West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-d881f713.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Florida West", "SU1-18-D-d881f713"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-c869a64f",
        title: "Mercator Data Georgia East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-c869a64f.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Georgia East", "SU1-18-D-c869a64f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-d7d659bf",
        title: "Mercator Data Georgia West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-d7d659bf.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Georgia West", "SU1-18-D-d7d659bf"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-82a51873",
        title: "Mercator Data Hawaii Zone 1",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-82a51873.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Hawaii Zone 1", "SU1-18-D-82a51873"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-a436f02b",
        title: "Mercator Data Hawaii Zone 2",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-a436f02b.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Hawaii Zone 2", "SU1-18-D-a436f02b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-047f92cf",
        title: "Mercator Data Hawaii Zone 3",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-047f92cf.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Hawaii Zone 3", "SU1-18-D-047f92cf"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-cdbe8da9",
        title: "Mercator Data Hawaii Zone 4",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-cdbe8da9.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Hawaii Zone 4", "SU1-18-D-cdbe8da9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-1f2efb54",
        title: "Mercator Data Hawaii Zone 5",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-1f2efb54.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Hawaii Zone 5", "SU1-18-D-1f2efb54"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-cd62b820",
        title: "Mercator Data Idaho Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-cd62b820.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Idaho Central", "SU1-18-D-cd62b820"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-12d9097b",
        title: "Mercator Data Idaho East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-12d9097b.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Idaho East", "SU1-18-D-12d9097b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-1f418de0",
        title: "Mercator Data Idaho West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-1f418de0.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Idaho West", "SU1-18-D-1f418de0"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-afb9a1d5",
        title: "Mercator Data Illinois East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-afb9a1d5.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Illinois East", "SU1-18-D-afb9a1d5"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-8a7bc9e3",
        title: "Mercator Data Illinois West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-8a7bc9e3.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Illinois West", "SU1-18-D-8a7bc9e3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-bff2bf90",
        title: "Mercator Data Indiana East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-bff2bf90.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Indiana East", "SU1-18-D-bff2bf90"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-faa6af3d",
        title: "Mercator Data Indiana West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-faa6af3d.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Indiana West", "SU1-18-D-faa6af3d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-2c492798",
        title: "Mercator Data Maine East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-2c492798.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Maine East", "SU1-18-D-2c492798"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-5fe3925b",
        title: "Mercator Data Maine West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-5fe3925b.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Maine West", "SU1-18-D-5fe3925b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-4e5aae3e",
        title: "Mercator Data Michigan Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-4e5aae3e.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Michigan Central", "SU1-18-D-4e5aae3e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-f4855de5",
        title: "Mercator Data Michigan East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-f4855de5.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Michigan East", "SU1-18-D-f4855de5"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-dec10f2a",
        title: "Mercator Data Michigan West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-dec10f2a.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Michigan West", "SU1-18-D-dec10f2a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-8895a531",
        title: "Mercator Data Mississippi East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-8895a531.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Mississippi East", "SU1-18-D-8895a531"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-d97a23b0",
        title: "Mercator Data Mississippi West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-d97a23b0.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Mississippi West", "SU1-18-D-d97a23b0"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-9cbae632",
        title: "Mercator Data Missouri Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-9cbae632.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Missouri Central", "SU1-18-D-9cbae632"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-594b90a3",
        title: "Mercator Data Missouri East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-594b90a3.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Missouri East", "SU1-18-D-594b90a3"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-4fcf10e8",
        title: "Mercator Data Missouri West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-4fcf10e8.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Missouri West", "SU1-18-D-4fcf10e8"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-59fa689d",
        title: "Mercator Data Nevada Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-59fa689d.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Nevada Central", "SU1-18-D-59fa689d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-bbf05970",
        title: "Mercator Data Nevada East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-bbf05970.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Nevada East", "SU1-18-D-bbf05970"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-9bf5465f",
        title: "Mercator Data Nevada West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-9bf5465f.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Nevada West", "SU1-18-D-9bf5465f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-739f5983",
        title: "Mercator Data New Hampshire",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-739f5983.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New Hampshire", "SU1-18-D-739f5983"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-afce7ace",
        title: "Mercator Data New Jersey",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-afce7ace.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New Jersey", "SU1-18-D-afce7ace"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-eefb46c0",
        title: "Mercator Data New Mexico Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-eefb46c0.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New Mexico Central", "SU1-18-D-eefb46c0"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-e63a1cc1",
        title: "Mercator Data New Mexico East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-e63a1cc1.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New Mexico East", "SU1-18-D-e63a1cc1"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-eb4cb1ea",
        title: "Mercator Data New Mexico West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-eb4cb1ea.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New Mexico West", "SU1-18-D-eb4cb1ea"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-9318ff8f",
        title: "Mercator Data New York Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-9318ff8f.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New York Central", "SU1-18-D-9318ff8f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-ec6a03a4",
        title: "Mercator Data New York East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-ec6a03a4.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New York East", "SU1-18-D-ec6a03a4"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-c17bcd0b",
        title: "Mercator Data New York West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-c17bcd0b.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data New York West", "SU1-18-D-c17bcd0b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-719a99d9",
        title: "Mercator Data Rhode Island",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-719a99d9.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Rhode Island", "SU1-18-D-719a99d9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-497a4397",
        title: "Mercator Data Vermont",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-497a4397.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Vermont", "SU1-18-D-497a4397"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-cdcd1f3a",
        title: "Mercator Data Wyoming East Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-cdcd1f3a.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Wyoming East Central", "SU1-18-D-cdcd1f3a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-e650d42f",
        title: "Mercator Data Wyoming East",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-e650d42f.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Wyoming East", "SU1-18-D-e650d42f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-3cce5d22",
        title: "Mercator Data Wyoming West Central",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-3cce5d22.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Wyoming West Central", "SU1-18-D-3cce5d22"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-18-D-43ef81bd",
        title: "Mercator Data Wyoming West",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-22.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-18-D-43ef81bd.hp67card"
        )],
        artwork: catalog_artwork("Mercator Data Wyoming West", "SU1-18-D-43ef81bd"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-19-D-b75b077e",
        title: "Geographic to Alaska 2-9",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-24.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-19-D-b75b077e.hp67card"
        )],
        artwork: catalog_artwork("Geographic to Alaska 2-9", "SU1-19-D-b75b077e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-19-02c58ff9",
        title: "19a - Geographic to Alaska 2-9",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-23.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-19-02c58ff9.hp67card"
        )],
        artwork: catalog_artwork("19a - Geographic to Alaska 2-9", "SU1-19-02c58ff9"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-19-D-c4b03467",
        title: "Alaska 2-9 to Geographic",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-26.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-19-D-c4b03467.hp67card"
        )],
        artwork: catalog_artwork("Alaska 2-9 to Geographic", "SU1-19-D-c4b03467"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Surveying Pac I",
        reference: "SU1-19-4ee33dbf",
        title: "19b - Alaska 2-9 to Geographic",
        source_pdf: Some("programs/HP67/ImportedPACs/manuals/hp6797-pac-surveying-en.pdf"),
        artwork_path: Some("assets/hp67-imported-artwork/surveying-25.png"),
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/surveying/SU1-19-4ee33dbf.hp67card"
        )],
        artwork: catalog_artwork("19b - Alaska 2-9 to Geographic", "SU1-19-4ee33dbf"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-A",
        title: "A LN Test",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-A.hp67card")],
        artwork: catalog_artwork("A LN Test", "DE-A"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-B",
        title: "B Base Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-B.hp67card")],
        artwork: catalog_artwork("B Base Conversion", "DE-B"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-C",
        title: "C TVMa",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-C.hp67card")],
        artwork: catalog_artwork("C TVMa", "DE-C"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-D",
        title: "D Factors Primes a",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-D.hp67card")],
        artwork: catalog_artwork("D Factors Primes a", "DE-D"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-E",
        title: "E Universal Converter",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-E.hp67card")],
        artwork: catalog_artwork("E Universal Converter", "DE-E"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-a",
        title: "F a Pressure data",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-a.hp67card")],
        artwork: catalog_artwork("F a Pressure data", "DE-F-a"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-b",
        title: "F b Distance data",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-b.hp67card")],
        artwork: catalog_artwork("F b Distance data", "DE-F-b"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-c",
        title: "F c Energy data",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-c.hp67card")],
        artwork: catalog_artwork("F c Energy data", "DE-F-c"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-d",
        title: "F d Mass data",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-d.hp67card")],
        artwork: catalog_artwork("F d Mass data", "DE-F-d"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-e",
        title: "F e Volume data hosghead",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-e.hp67card")],
        artwork: catalog_artwork("F e Volume data hosghead", "DE-F-e"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-F-f",
        title: "F f Volume data peck",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-F-f.hp67card")],
        artwork: catalog_artwork("F f Volume data peck", "DE-F-f"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-G",
        title: "G PVnRT",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-G.hp67card")],
        artwork: catalog_artwork("G PVnRT", "DE-G"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-H",
        title: "H Blood Alcohol",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-H.hp67card")],
        artwork: catalog_artwork("H Blood Alcohol", "DE-H"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-I",
        title: "I Weak Acid Titration",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-I.hp67card")],
        artwork: catalog_artwork("I Weak Acid Titration", "DE-I"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-J",
        title: "J Weak Base Titration",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-J.hp67card")],
        artwork: catalog_artwork("J Weak Base Titration", "DE-J"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-K",
        title: "K Is It Prime",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-K.hp67card")],
        artwork: catalog_artwork("K Is It Prime", "DE-K"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-L",
        title: "L Multiple Linear Regression A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-L.hp67card")],
        artwork: catalog_artwork("L Multiple Linear Regression A", "DE-L"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-M",
        title: "M Curve Fitting A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-M.hp67card")],
        artwork: catalog_artwork("M Curve Fitting A", "DE-M"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-N",
        title: "N Sunrise Sunset Time A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-N.hp67card")],
        artwork: catalog_artwork("N Sunrise Sunset Time A", "DE-N"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-O",
        title: "O Base Conversion 1 a",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-O.hp67card")],
        artwork: catalog_artwork("O Base Conversion 1 a", "DE-O"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-X",
        title: "X Diagnostic 15A a",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-X.hp67card")],
        artwork: catalog_artwork("X Diagnostic 15A a", "DE-X"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-Y",
        title: "Y Diagnostic 15B a",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-Y.hp67card")],
        artwork: catalog_artwork("Y Diagnostic 15B a", "DE-Y"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Pac",
        reference: "DE-Z",
        title: "Z Diagnostic 15Ca",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!("/programs/HP67/ImportedPACs/eaton/DE-Z.hp67card")],
        artwork: catalog_artwork("Z Diagnostic 15Ca", "DE-Z"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A00",
        title: "A00-Time Value of Money",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A00.hp67card"
        )],
        artwork: catalog_artwork("A00-Time Value of Money", "DE26-A00"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A02",
        title: "A02-Weak acid Strong base titration",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A02.hp67card"
        )],
        artwork: catalog_artwork("A02-Weak acid Strong base titration", "DE26-A02"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A03",
        title: "A03-Weak base Strong acid titration",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A03.hp67card"
        )],
        artwork: catalog_artwork("A03-Weak base Strong acid titration", "DE26-A03"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A04",
        title: "A04-Ideal gas PVnRT",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A04.hp67card"
        )],
        artwork: catalog_artwork("A04-Ideal gas PVnRT", "DE26-A04"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A05",
        title: "A05-Pressure Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A05.hp67card"
        )],
        artwork: catalog_artwork("A05-Pressure Conversion", "DE26-A05"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A07",
        title: "A07-Distance Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A07.hp67card"
        )],
        artwork: catalog_artwork("A07-Distance Conversion", "DE26-A07"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A09",
        title: "A09-Power Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A09.hp67card"
        )],
        artwork: catalog_artwork("A09-Power Conversion", "DE26-A09"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A11",
        title: "A11-Mass Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A11.hp67card"
        )],
        artwork: catalog_artwork("A11-Mass Conversion", "DE26-A11"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A13",
        title: "A13-Volume Conversion 1",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A13.hp67card"
        )],
        artwork: catalog_artwork("A13-Volume Conversion 1", "DE26-A13"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A15",
        title: "A15-Volume Conversion 2",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A15.hp67card"
        )],
        artwork: catalog_artwork("A15-Volume Conversion 2", "DE26-A15"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A17",
        title: "A17-Factors of N",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A17.hp67card"
        )],
        artwork: catalog_artwork("A17-Factors of N", "DE26-A17"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A19",
        title: "A19-Is it Prime",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A19.hp67card"
        )],
        artwork: catalog_artwork("A19-Is it Prime", "DE26-A19"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A20",
        title: "A20-Base Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A20.hp67card"
        )],
        artwork: catalog_artwork("A20-Base Conversion", "DE26-A20"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A21",
        title: "A21-Ohm's Law",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A21.hp67card"
        )],
        artwork: catalog_artwork("A21-Ohm's Law", "DE26-A21"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A22",
        title: "A22-Reactance Calc",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A22.hp67card"
        )],
        artwork: catalog_artwork("A22-Reactance Calc", "DE26-A22"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A24",
        title: "A24-Series Caps Parallel Resistor",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A24.hp67card"
        )],
        artwork: catalog_artwork("A24-Series Caps Parallel Resistor", "DE26-A24"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A25",
        title: "A25-Quadratic Equation",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A25.hp67card"
        )],
        artwork: catalog_artwork("A25-Quadratic Equation", "DE26-A25"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A26",
        title: "A26-Lin Regress",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A26.hp67card"
        )],
        artwork: catalog_artwork("A26-Lin Regress", "DE26-A26"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A27",
        title: "A27-Tempertaure Conversion",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A27.hp67card"
        )],
        artwork: catalog_artwork("A27-Tempertaure Conversion", "DE26-A27"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-A28",
        title: "A28-Curve Fitting 1",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-A28.hp67card"
        )],
        artwork: catalog_artwork("A28-Curve Fitting 1", "DE26-A28"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E00",
        title: "E00-Diagnostic 15A",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E00.hp67card"
        )],
        artwork: catalog_artwork("E00-Diagnostic 15A", "DE26-E00"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E02",
        title: "E02-Diagnostic 15B",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E02.hp67card"
        )],
        artwork: catalog_artwork("E02-Diagnostic 15B", "DE26-E02"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E04",
        title: "E04-Diagnostic 15C",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E04.hp67card"
        )],
        artwork: catalog_artwork("E04-Diagnostic 15C", "DE26-E04"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E06",
        title: "E06-Diagnostic 15A Loop",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E06.hp67card"
        )],
        artwork: catalog_artwork("E06-Diagnostic 15A Loop", "DE26-E06"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E08",
        title: "E08-Diagnostic 15B Loop",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E08.hp67card"
        )],
        artwork: catalog_artwork("E08-Diagnostic 15B Loop", "DE26-E08"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E10",
        title: "E10-Diagnostic 15C Loop",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E10.hp67card"
        )],
        artwork: catalog_artwork("E10-Diagnostic 15C Loop", "DE26-E10"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E12",
        title: "E12-Diagnostic 15A Loop No Print",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E12.hp67card"
        )],
        artwork: catalog_artwork("E12-Diagnostic 15A Loop No Print", "DE26-E12"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E14",
        title: "E14-Diagnostic 15B Loop No Print",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E14.hp67card"
        )],
        artwork: catalog_artwork("E14-Diagnostic 15B Loop No Print", "DE26-E14"),
    },
    ProgramLibraryEntry {
        pack: "HP-67 Dave Eaton Supplementary Pac (2026)",
        reference: "DE26-E16",
        title: "E16-Diagnostic 15C Loop No Print",
        source_pdf: None,
        artwork_path: None,
        parts: &[hpp!(
            "/programs/HP67/ImportedPACs/eaton2026/DE26-E16.hp67card"
        )],
        artwork: catalog_artwork("E16-Diagnostic 15C Loop No Print", "DE26-E16"),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabetical_display_order_keeps_every_original_catalog_index() {
        let order = program_library_display_order();
        let mut indices = order.to_vec();
        indices.sort_unstable();
        assert_eq!(indices, (0..PROGRAM_LIBRARY.len()).collect::<Vec<_>>());
        for pair in order.windows(2) {
            let a = &PROGRAM_LIBRARY[pair[0]];
            let b = &PROGRAM_LIBRARY[pair[1]];
            assert!(
                (a.pack.to_lowercase(), a.title.to_lowercase(), a.reference)
                    <= (b.pack.to_lowercase(), b.title.to_lowercase(), b.reference)
            );
        }
    }

    #[test]
    fn pack_names_use_consistent_hp67_prefix() {
        for entry in PROGRAM_LIBRARY {
            assert!(entry.pack.starts_with("HP-67 "), "{}", entry.pack);
        }
    }

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
        assert_eq!(rows.len(), 349);
        rows.sort_unstable();
        rows.dedup();
        assert_eq!(rows, (0..349).collect::<Vec<_>>());
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
    fn catalog_contains_all_built_in_program_media() {
        assert_eq!(PROGRAM_LIBRARY.len(), 438);
        assert_eq!(
            PROGRAM_LIBRARY
                .iter()
                .map(ProgramLibraryEntry::track_count)
                .sum::<usize>(),
            790
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
            .filter(|entry| entry.pack == "HP-67 Custom Diagnostic Pac")
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 12);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.reference)
                .collect::<Vec<_>>(),
            [
                "CD-01", "CD-02", "CD-03", "CD-04", "CD-05", "CD-06", "CD-07", "CD-08", "CD-09",
                "CD-10", "CD-11", "CD-12",
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
    fn custom_diagnostic_library_entries_use_native_hp67card() {
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
            assert_eq!(entry.parts.len(), 1, "{reference} must use one native card");
            assert!(
                entry.parts[0].starts_with(b"HP67CARD"),
                "{reference} must not depend on the Teenix compatibility format"
            );
            let loaded = entry.load_card().unwrap().card;
            assert_eq!(native, loaded, "{reference} native media mismatch");
        }
    }

    #[test]
    fn standard_diagnostic_library_uses_native_card() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD1-15A")
            .expect("Standard Pac diagnostic must be in the catalog");
        assert_eq!(entry.parts.len(), 1);
        assert!(entry.parts[0].starts_with(b"HP67CARD"));
        let imported = entry.load_card().unwrap().card;
        let native = Hp67MagneticCard::from_hp67card_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/HP-67 Standard Pac/SD1-15A-Diagnostic-Program.hp67card"
        )))
        .expect("checked-in native Standard diagnostic must decode");
        assert_eq!(native, imported);
    }

    #[test]
    fn exact_sd15c_library_entry_uses_native_card() {
        let entry = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD-15C")
            .expect("SD-15C must be in the catalog");
        assert_eq!(entry.parts.len(), 1);
        assert!(entry.parts[0].starts_with(b"HP67CARD"));
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
