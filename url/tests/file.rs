use url::Url;

fn assert_file_url(input: &str, base: Option<&str>, expected: &str) {
    let base = base.map(|value| Url::parse(value).unwrap());
    let url = Url::options().base_url(base.as_ref()).parse(input).unwrap();
    assert_eq!(url.as_str(), expected, "input {input:?}, base {base:?}");
    assert_eq!(Url::parse(url.as_str()).unwrap(), url);
    assert_eq!(url.username(), "");
    assert_eq!(url.password(), None);
    assert_eq!(url.port(), None);
}

#[test]
fn preserve_empty_path_segments() {
    for (input, expected) in [
        ("file://server///", "file://server///"),
        ("file:////one/two", "file:////one/two"),
        ("file://localhost////foo", "file://////foo"),
        (r"file:\\\\", "file:////"),
        ("file://localhost//a//../..//foo", "file://///foo"),
        ("file:.//p", "file:////p"),
        ("file:/.//p", "file:////p"),
        ("file:////?q#f", "file:////?q#f"),
    ] {
        assert_file_url(input, None, expected);
    }
    assert_file_url("/////mouse", Some("file:///elephant"), "file://///mouse");
    assert_file_url(r"\/localhost//pig", Some("file://lion/"), "file:////pig");
}

#[test]
fn empty_host_enters_path_start_state() {
    assert_file_url("file:///w|/m", None, "file:///w:/m");
    assert_file_url("file://localhost/w|/m", None, "file:///w:/m");
    assert_file_url("file://w|/m", None, "file:///w:/m");
    assert_file_url("file:////w|/m", None, "file:////w|/m");
}

#[test]
fn path_mutation_preserves_empty_segments() {
    for (input, path, expected) in [
        ("file:///", "//foo", "file:////foo"),
        ("file://monkey/", r"\\", "file://monkey//"),
        ("file:///unicorn", r"//\/", "file://////"),
        ("file:///unicorn", "//monkey/..//", "file://///"),
    ] {
        let mut url = Url::parse(input).unwrap();
        url.set_path(path);
        assert_eq!(url.as_str(), expected);
        assert_eq!(Url::parse(url.as_str()).unwrap(), url);
    }

    let mut url = Url::parse("file:////?q#f").unwrap();
    url.path_segments_mut().unwrap().push("foo");
    assert_eq!(url.as_str(), "file://///foo?q#f");
    assert_eq!(Url::parse(url.as_str()).unwrap(), url);
}

#[test]
fn drive_letters_preserve_explicit_hosts() {
    for host in ["", "server", "127.0.0.1", "[::1]"] {
        for path in ["C:/", "C|/", "C|", "C|?q#f", "C:/dir/file"] {
            let input = format!("file://{host}/{path}");
            let expected = format!("file://{host}/{}", path.replacen('|', ":", 1));
            assert_file_url(&input, None, &expected);
        }
    }

    let violation = std::cell::Cell::new(None);
    let url = Url::options()
        .syntax_violation_callback(Some(&|value| violation.set(Some(value))))
        .parse("file://server/C:/path")
        .unwrap();
    assert_eq!(url.as_str(), "file://server/C:/path");
    assert_eq!(violation.get(), None);

    let mut url = Url::parse("file://host/old?q#f").unwrap();
    url.set_path("C|/new");
    assert_eq!(url.as_str(), "file://host/C:/new?q#f");
    assert_eq!(Url::parse(url.as_str()).unwrap(), url);
}

#[test]
fn relative_paths_inherit_hosts_independently_of_drive_letters() {
    for (input, base, expected) in [
        ("/", "file://h/C:/a/b", "file://h/C:/"),
        ("/next", "file://h/C:/a/b", "file://h/C:/next"),
        ("C|", "file://host/dir/file", "file://host/C:"),
        ("C|/", "file://host/D:/dir/file", "file://host/C:/"),
        ("C|\n/", "file://host/dir/file", "file://host/C:/"),
        (r"C|\", "file://host/dir/file", "file://host/C:/"),
        ("/c:/foo/bar", "file://host/path", "file://host/c:/foo/bar"),
        ("file:C:/", "file://host/", "file://host/C:/"),
        ("file:/C:/", "file://host/", "file://host/C:/"),
    ] {
        assert_file_url(input, Some(base), expected);
    }
}
