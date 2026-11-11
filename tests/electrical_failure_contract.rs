//! Public dense-fabric error semantics: publication is not rolled back.
use hp67emu::emulation::{Drive, LogicLevel, Tick};
use hp67emu::machines::hp67::{Hp67Driver, Hp67ElectricalError, Hp67ElectricalFabric, Hp67Net};

#[test]
fn core_audit_failed_commit_publishes_complete_image_without_advancing_time() {
    let mut fabric = Hp67ElectricalFabric::default();
    let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
    let rom = fabric
        .claim_driver_owner(Hp67Driver::StructuralRomResponder)
        .unwrap();
    fabric.stage_drive(&act, Hp67Net::Data, Drive::High);
    fabric.stage_drive(&rom, Hp67Net::Data, Drive::Low);
    // This later, independent proposal must also be committed before reporting
    // the resulting contention. A caller must not assume Err means rollback.
    fabric.stage_drive(&act, Hp67Net::Sync, Drive::High);
    assert!(matches!(
        fabric.commit_staged(),
        Err(Hp67ElectricalError::Contention {
            net: Hp67Net::Data,
            ..
        })
    ));
    assert_eq!(fabric.tick(), Tick::ZERO);
    assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Contention);
    assert_eq!(fabric.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(
        fabric.drive(Hp67Net::Data, Hp67Driver::Act1820_2530),
        Drive::High
    );
    assert_eq!(
        fabric.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
        Drive::Low
    );
    assert!(fabric.begin_evaluation().is_err());
    // Explicit low-level fault recovery releases the offending contributor.
    // A subsequent empty commit must not replay the failed batch.
    fabric.set_drive_immediate(
        Hp67Net::Data,
        Hp67Driver::StructuralRomResponder,
        Drive::HighZ,
    );
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

#[test]
fn core_audit_foreign_driver_cannot_replace_pending_output() {
    let driver = Hp67Driver::Act1820_2530;
    let mut first = Hp67ElectricalFabric::default();
    let foreign = first.claim_driver_owner(driver).unwrap();
    let mut second = Hp67ElectricalFabric::default();
    let local = second.claim_driver_owner(driver).unwrap();
    assert_ne!(foreign, local);

    second.stage_drive(&local, Hp67Net::Sync, Drive::High);
    {
        let (snapshot, mut stager) = second.begin_evaluation().unwrap();
        assert_eq!(snapshot.level(Hp67Net::Sync), LogicLevel::Floating);
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            stager.stage_drive(&foreign, Hp67Net::Sync, Drive::Low);
        }));
        assert!(rejected.is_err());
    }
    second.commit_staged().unwrap();
    assert_eq!(second.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(second.tick(), Tick::ZERO);
}

#[test]
fn core_audit_cloned_fabric_keeps_electrical_state_but_requires_new_driver_claims() {
    let driver = Hp67Driver::Act1820_2530;
    let mut original = Hp67ElectricalFabric::default();
    let owner = original.claim_driver_owner(driver).unwrap();
    original.stage_drive(&owner, Hp67Net::Sync, Drive::High);

    let mut cloned = original.clone();
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cloned.stage_drive(&owner, Hp67Net::Sync, Drive::Low);
    }));
    assert!(rejected.is_err());
    cloned.commit_staged().unwrap();
    assert_eq!(cloned.level(Hp67Net::Sync), LogicLevel::High);
    let cloned_owner = cloned.claim_driver_owner(driver).unwrap();
    cloned.stage_drive(&cloned_owner, Hp67Net::Sync, Drive::Low);
    cloned.commit_staged().unwrap();

    let mut moved = original;
    moved.stage_drive(&owner, Hp67Net::Sync, Drive::High);
    moved.commit_staged().unwrap();
    assert_eq!(moved.level(Hp67Net::Sync), LogicLevel::High);
    assert_eq!(cloned.level(Hp67Net::Sync), LogicLevel::Low);
}
