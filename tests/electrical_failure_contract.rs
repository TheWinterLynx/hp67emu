//! Public dense-fabric error semantics: publication is not rolled back.
use hp67emu::emulation::{Drive, LogicLevel, Tick};
use hp67emu::machines::hp67::{
    Hp67Driver, Hp67ElectricalError, Hp67ElectricalFabric, Hp67Net,
};

#[test]
fn core_audit_failed_commit_publishes_complete_image_without_advancing_time() {
    let mut fabric = Hp67ElectricalFabric::default();
    let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
    let rom = fabric.claim_driver_owner(Hp67Driver::StructuralRomResponder).unwrap();
    fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
    fabric.stage_drive(&rom, Hp67Net::Data, Drive::Low);
    // This later, independent proposal must also be committed before reporting
    // the resulting contention. A caller must not assume Err means rollback.
    fabric.stage_drive(&act, Hp67Net::Sync, Drive::High);
    assert!(matches!(
        fabric.commit_staged(),
        Err(Hp67ElectricalError::Contention { net: Hp67Net::Data, .. })
    ));
    assert_eq!(fabric.tick(), Tick::ZERO);
    assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Contention);
    assert_eq!(fabric.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(fabric.drive(Hp67Net::Data, Hp67Driver::Act1820_2530), Drive::High);
    assert_eq!(fabric.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder), Drive::Low);
    assert!(fabric.begin_evaluation().is_err());
    // Explicit low-level fault recovery releases the offending contributor.
    // A subsequent empty commit must not replay the failed batch.
    fabric.set_drive_immediate(Hp67Net::Data, Hp67Driver::StructuralRomResponder, Drive::HighZ);
    fabric.commit_staged().unwrap();
    assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
    assert_eq!(fabric.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(fabric.tick(), Tick::ZERO);
}

#[test]
fn core_audit_rejected_evaluation_preserves_pending_outputs_until_explicit_recovery() {
    let mut fabric = Hp67ElectricalFabric::default();
    let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
    fabric.stage_drive(&act, Hp67Net::Sync, Drive::High);
    fabric.set_drive_immediate(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
    fabric.set_drive_immediate(Hp67Net::Data, Hp67Driver::External, Drive::Low);
    assert!(fabric.begin_evaluation().is_err());
    assert_eq!(fabric.level(Hp67Net::Sync), LogicLevel::Floating);
    fabric.set_drive_immediate(Hp67Net::Data, Hp67Driver::External, Drive::HighZ);
    fabric.commit_staged().unwrap();
    assert_eq!(fabric.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(fabric.tick(), Tick::ZERO);
}
