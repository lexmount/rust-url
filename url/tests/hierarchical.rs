use url::Url;

fn assert_url(url: &Url, expected: &str, path: &str) {
    assert_eq!(url.as_str(), expected);
    assert_eq!(url.path(), path);
    url.check_invariants().unwrap();
    assert_eq!(Url::parse(url.as_str()).unwrap(), *url);
}

#[test]
fn relative_special_urls_skip_all_authority_slashes() {
    for scheme in ["http", "https", "ws", "wss", "ftp"] {
        let base = Url::parse(&format!("{scheme}://user:pass@old.test:123/a/b?q#f")).unwrap();
        for prefix in ["//", "///", r"///\//\//", r"\\", r"/\", r"\/", "/\t/\n/"] {
            let url = base
                .join(&format!("{prefix}new.test/a/../p?next#frag"))
                .unwrap();
            assert_url(&url, &format!("{scheme}://new.test/p?next#frag"), "/p");
            assert_eq!(url.host_str(), Some("new.test"));
            assert_eq!(url.port(), None);
            assert!(url.username().is_empty());
            assert_eq!(url.password(), None);
        }
        for input in ["///", r"/\\", "////?query", r"\\#fragment"] {
            assert!(base.join(input).is_err(), "{}: {:?}", scheme, input);
        }
        for input in ["/p", r"\p"] {
            assert_url(
                &base.join(input).unwrap(),
                &format!("{scheme}://user:pass@old.test:123/p"),
                "/p",
            );
        }
    }
}

#[test]
fn relative_non_special_urls_keep_backslashes_in_the_path() {
    for (base, prefix) in [
        ("custom://host/a/b?q#f", "custom://host/a/"),
        ("custom:/a/b?q#f", "custom:/a/"),
        ("custom:/.//a/b?q#f", "custom:/.//a/"),
    ] {
        let base = Url::parse(base).unwrap();
        for input in [r"\p", r"\/p", r"\\p"] {
            let expected = format!("{prefix}{input}");
            let url = base.join(input).unwrap();
            assert_eq!(url.as_str(), expected);
            url.check_invariants().unwrap();
            assert_eq!(url.has_authority(), base.has_authority());
            assert_eq!(url.host_str(), base.host_str());
        }
        for (input, expected, path) in [
            ("//next", "custom://next", ""),
            ("///next", "custom:///next", "/next"),
            ("////next", "custom:////next", "//next"),
        ] {
            assert_url(&base.join(input).unwrap(), expected, path);
        }
        assert!(base.join(r"//\next").is_err());
    }
}

#[test]
fn hierarchical_path_setters_add_and_remove_only_the_serialization_prefix() {
    for input in ["custom:/old?q#f", "custom:/.//old?q#f"] {
        for (value, expected, path) in [
            ("//p", "custom:/.//p?q#f", "//p"),
            ("/.//p", "custom:/.//p?q#f", "//p"),
            ("/..//p", "custom:/.//p?q#f", "//p"),
            ("//", "custom:/.//?q#f", "//"),
            ("p", "custom:/p?q#f", "/p"),
            ("/", "custom:/?q#f", "/"),
        ] {
            let mut url = Url::parse(input).unwrap();
            url.set_path(value);
            assert_url(&url, expected, path);
            assert!(!url.has_authority());
            assert_eq!(url.query(), Some("q"));
            assert_eq!(url.fragment(), Some("f"));
        }
        let mut url = Url::parse(input).unwrap();
        url::quirks::set_pathname(&mut url, "");
        assert_url(&url, "custom:/?q#f", "/");
    }
    for authority in ["", "host", "user:pass@host:123"] {
        let mut url = Url::parse(&format!("custom://{authority}/old?q#f")).unwrap();
        url.set_path("//p");
        assert_url(&url, &format!("custom://{authority}//p?q#f"), "//p");
        assert!(url.has_authority());
    }
}

#[test]
fn path_segment_mutations_keep_the_path_prefix_and_component_offsets_consistent() {
    let mut url = Url::parse("custom:/.//p?q#f").unwrap();
    url.path_segments_mut().unwrap().pop();
    assert_url(&url, "custom:/?q#f", "/");
    url.set_path("//p");
    url.path_segments_mut()
        .unwrap()
        .clear()
        .push("next")
        .push("");
    assert_url(&url, "custom:/next/?q#f", "/next/");
    url.set_path("///");
    url.path_segments_mut().unwrap().pop_if_empty();
    assert_url(&url, "custom:/.//?q#f", "//");
    url.path_segments_mut().unwrap().pop_if_empty();
    assert_url(&url, "custom:/?q#f", "/");
}
