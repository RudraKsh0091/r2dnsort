//! <- test_ns_enum.py

use r2dnsort::ns_enum::Ns;

#[test]
fn test_ns_enum_values_and_aliases() {
    assert_eq!(Ns::FLOAT.raw(), 0x0001);
    assert_eq!(Ns::SIGNED.raw(), 0x0002);
    assert_eq!(Ns::NOEXP.raw(), 0x0004);
    assert_eq!(Ns::PATH.raw(), 0x0008);
    assert_eq!(Ns::LOCALEALPHA.raw(), 0x0010);
    assert_eq!(Ns::LOCALENUM.raw(), 0x0020);
    assert_eq!(Ns::IGNORECASE.raw(), 0x0040);
    assert_eq!(Ns::LOWERCASEFIRST.raw(), 0x0080);
    assert_eq!(Ns::GROUPLETTERS.raw(), 0x0100);
    assert_eq!(Ns::UNGROUPLETTERS.raw(), 0x0200);
    assert_eq!(Ns::NANLAST.raw(), 0x0400);
    assert_eq!(Ns::COMPATIBILITYNORMALIZE.raw(), 0x0800);
    assert_eq!(Ns::NUMAFTER.raw(), 0x1000);
    assert_eq!(Ns::PRESORT.raw(), 0x2000);
    assert_eq!(Ns::DEFAULT.raw(), 0x0000);
    assert_eq!(Ns::INT.raw(), 0x0000);
    assert_eq!(Ns::UNSIGNED.raw(), 0x0000);
    assert_eq!(Ns::REAL.raw(), 0x0003);
    assert_eq!(Ns::LOCALE.raw(), 0x0030);
    assert_eq!(Ns::I.raw(), 0x0000);
    assert_eq!(Ns::U.raw(), 0x0000);
    assert_eq!(Ns::F.raw(), 0x0001);
    assert_eq!(Ns::S.raw(), 0x0002);
    assert_eq!(Ns::R.raw(), 0x0003);
    assert_eq!(Ns::N.raw(), 0x0004);
    assert_eq!(Ns::P.raw(), 0x0008);
    assert_eq!(Ns::LA.raw(), 0x0010);
    assert_eq!(Ns::LN.raw(), 0x0020);
    assert_eq!(Ns::L.raw(), 0x0030);
    assert_eq!(Ns::IC.raw(), 0x0040);
    assert_eq!(Ns::LF.raw(), 0x0080);
    assert_eq!(Ns::G.raw(), 0x0100);
    assert_eq!(Ns::UG.raw(), 0x0200);
    assert_eq!(Ns::C.raw(), 0x0200);
    assert_eq!(Ns::CAPITALFIRST.raw(), 0x0200);
    assert_eq!(Ns::NL.raw(), 0x0400);
    assert_eq!(Ns::CN.raw(), 0x0800);
    assert_eq!(Ns::NA.raw(), 0x1000);
    assert_eq!(Ns::PS.raw(), 0x2000);
}
