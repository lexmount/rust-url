use url::Url;

fn assert_url(url: &Url, expected: &str, path: &str) {
    assert_eq!(url.as_str(), expected);
    assert_eq!(url.path(), path);
    url.check_invariants().unwrap();
    assert_eq!(Url::parse(url.as_str()).unwrap(), *url);
}

#[test]
fn opaque_paths_encode_only_the_last_space_before_a_query_or_fragment() {
    for scheme in ["data", "mailto", "non-special"] {
        for (input, path) in [
            ("payload ", "payload%20"),
            ("payload   ", "payload  %20"),
            ("payload \t\r\n", "payload%20"),
            ("payload \t \n \r", "payload  %20"),
            (" ", "%20"),
            ("payload%20", "payload%20"),
            ("payload \u{a0}", "payload %C2%A0"),
            ("payload %3F", "payload %3F"),
        ] {
            for suffix in ["?", "#", "?q", "#h", "?q#h", "?#"] {
                let url = Url::parse(&format!("{scheme}:{input}{suffix}")).unwrap();
                assert_url(&url, &format!("{scheme}:{path}{suffix}"), path);
                assert!(url.cannot_be_a_base());
            }
        }
    }
}

#[test]
fn opaque_paths_preserve_interior_spaces_carets_and_encoded_delimiters() {
    for (input, expected, path) in [
        ("data:a b^c?^#^", "data:a b^c?^#^", "a b^c"),
        ("data:a %3F %23?^#^", "data:a %3F %23?^#^", "a %3F %23"),
        ("data:a%20?^#^", "data:a%20?^#^", "a%20"),
        (" \tdata:a b^c   \r\n", "data:a b^c", "a b^c"),
        ("data:space ?a #b ", "data:space%20?a%20#b", "space%20"),
    ] {
        assert_url(&Url::parse(input).unwrap(), expected, path);
    }
}

#[test]
fn clearing_query_and_fragment_preserves_the_encoded_opaque_path_in_either_order() {
    for input in ["data:payload   ?q#h", "data:payload  %20?q#h"] {
        for query_first in [false, true] {
            let mut url = Url::parse(input).unwrap();
            if query_first {
                url.set_query(None);
                assert_url(&url, "data:payload  %20#h", "payload  %20");
                url.set_fragment(None);
            } else {
                url.set_fragment(None);
                assert_url(&url, "data:payload  %20?q", "payload  %20");
                url.set_query(None);
            }
            assert_url(&url, "data:payload  %20", "payload  %20");
            assert_url(
                &url.join("#next").unwrap(),
                "data:payload  %20#next",
                "payload  %20",
            );
            url.set_query(Some(""));
            url.set_fragment(Some(""));
            assert_url(&url, "data:payload  %20?#", "payload  %20");
            url.query_pairs_mut().clear().append_pair("next", "value");
            assert_url(&url, "data:payload  %20?next=value#", "payload  %20");
        }
    }
}
