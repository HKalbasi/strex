use crate::StrexSet;

#[test]
fn very_basic() {
    let strex_set = StrexSet::new([
        "salam.*aleyk",
        "foo.*bar",
        "(salam|hello).*foo",
        "(sa.*lam|hello).*foo",
        "(sa.*lam|hello).*(fooo|salam)",
        "salam.*aleyk.*ey.*foo",
    ]);
    let haystack = "salam aleyk ey foo";
    let mut matches = strex_set.matches(haystack).map(|x| x.0).collect::<Vec<_>>();
    matches.sort();
    assert_eq!(matches, [0, 2, 3, 5]);
}

#[test]
fn aabb_1() {
    let strex_set = StrexSet::new([
        "aaba.*abba.*bab",
        "aaba.*baa.*aabb.*bbb",
        "abaa.*bbaa.*bbba",
        "bbb.*baaa.*baaa.*aaab",
        "aaab.*babb.*bbb",
    ]);
    let haystack = "aababbbbabbbbaabbabaaabbbbbbbbbbabaaaabb";
    let mut matches = strex_set.matches(haystack).map(|x| x.0).collect::<Vec<_>>();
    matches.sort();
    assert_eq!(matches, [0, 1]);
}

#[test]
fn aabb_2() {
    let strex_set = StrexSet::new([
        "bab.*bbba.*aaaa",
        "aaba.*aba.*baaa.*abaa",
        "aaaa.*aba.*abba.*aab",
        "aab.*bbb.*bbb",
        "abba.*baa.*bbab",
    ]);
    let haystack = "aabbbababaaabbbbaabbbbaaaabbaabbabaabbbb";
    let mut matches = strex_set.matches(haystack).map(|x| x.0).collect::<Vec<_>>();
    matches.sort();
    assert_eq!(matches, [0, 3]);
}
