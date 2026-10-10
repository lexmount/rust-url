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
