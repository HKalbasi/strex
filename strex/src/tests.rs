use crate::StrexSet;

#[test]
fn very_basic() {
    let strex_set = StrexSet::new(["salam.*aleyk", "foo.*bar", "(salam|hello).*foo"]);
    let haystack = "salam aleyk ey foo";
    let matches = strex_set.matches(haystack).map(|x| x.0).collect::<Vec<_>>();
    assert_eq!(matches, [0, 2]);
}